#![cfg_attr(not(feature = "std"), no_std)]

use account_set::AccountSet;
use frame_support::storage::IterableStorageMap;
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, dispatch, dispatch::DispatchResult, ensure,
};
use frame_system::{self as system, ensure_signed};
use sp_std::collections::btree_set::BTreeSet;
use sp_std::prelude::*;

use bluefn_util::catalog::ProductId;
use bluefn_util::shipment::*;

#[cfg(test)]
mod tests;

/// A maximum number of members. When membership reaches this number, no new members may join.
pub const MAX_MEMBERS: u32 = 16;

pub trait Trait: system::Trait + timestamp::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as VecMap {
		//Currently we map to '()' because '()' is not encoded anymore as 0 bytes and the underlying storage
		Members get(fn members): map hasher(blake2_128_concat) T::AccountId => ();
		// The total number of members stored in the map.
		// Because the map does not store its size internally, we must store it separately
		MemberCount: u32;

		pub Shipments get(fn shipment_by_id): map hasher(blake2_128_concat) ShipmentId => Option<Shipment<T::AccountId, T::Moment>>;
        pub ShipmentsOfOrganization get(fn shipments_of_org): map hasher(blake2_128_concat) T::AccountId => Vec<ShipmentId>;

        pub EventCount get(fn event_count): u64;
        pub AllEvents get(fn event_by_idx): map hasher(blake2_128_concat) ShippingEventIndex => Option<ShippingEvent<T::Moment>>;
        pub EventIndices get(fn event_idx_from_id): map hasher(blake2_128_concat) ShippingEventId => Option<ShippingEventIndex>;
        pub EventsOfShipment get(fn events_of_shipment): map hasher(blake2_128_concat) ShipmentId => Vec<ShippingEventIndex>;
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		/// Added a member
		MemberAdded(AccountId),
		/// Removed a member
		MemberRemoved(AccountId),

		ShipmentRegistered(AccountId, ShipmentId, AccountId),
        ShipmentStatusUpdated(ShipmentId, ShipmentStatus),
        ShippingEventRecorded(AccountId, ShippingEventId, ShipmentId, ShippingEventType),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Cannot join as a member because you are already a member
		AlreadyMember,
		/// Cannot give up membership because you are not currently a member
		NotMember,
		/// Cannot add another member because the limit is already reached
		MembershipLimitReached,

		InvalidOrMissingIdentifier,
        ShipmentAlreadyExists,
        ShipmentHasBeenDelivered,
        ShipmentIsInTransit,
        ShipmentIsUnknown,
        ShipmentHasTooManyProducts,
        ShippingEventAlreadyExists,
        ShippingEventMaxExceeded,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		type Error = Error<T>;

		/// Adds a member to the membership set
		#[weight = 10_000]
		fn add_member(origin) -> DispatchResult {
			let new_member = ensure_signed(origin)?;

			let member_count = MemberCount::get();
			ensure!(member_count < MAX_MEMBERS, Error::<T>::MembershipLimitReached);

			// We don't want to add duplicate members, so we check whether the potential new
			// member is already present in the list. Because the membership is stored as a hash
			// map this check is constant time O(1)
			ensure!(!Members::<T>::contains_key(&new_member), Error::<T>::AlreadyMember);

			// Insert the new member and emit the event
			Members::<T>::insert(&new_member, ());
			MemberCount::put(member_count + 1); // overflow check not necessary because of maximum
			Self::deposit_event(RawEvent::MemberAdded(new_member));
			Ok(())
		}

		/// Removes a member.
		#[weight = 10_000]
		fn remove_member(origin) -> DispatchResult {
			let old_member = ensure_signed(origin)?;

			ensure!(Members::<T>::contains_key(&old_member), Error::<T>::NotMember);

			Members::<T>::remove(&old_member);
			MemberCount::mutate(|v| *v -= 1);
			Self::deposit_event(RawEvent::MemberRemoved(old_member));
			Ok(())
		}

		#[weight = 10_000]
        pub fn register_shipment(origin, id: ShipmentId, owner: T::AccountId, products: Vec<ProductId>) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;

            // TODO: assuming owner is a DID representing an organization,
            //       validate tx sender is owner or delegate of organization.

            // Validate format of shipment ID
            Self::validate_identifier(&id)?;

            // Validate shipment products
            Self::validate_shipment_products(&products)?;

            // Check shipment doesn't exist yet (1 DB read)
            Self::validate_new_shipment(&id)?;

            // Create a shipment instance
            let shipment = Self::new_shipment()
                .identified_by(id.clone())
                .owned_by(owner.clone())
                .registered_on(<timestamp::Module<T>>::now())
                .with_products(products)
                .build();
            let status = shipment.status.clone();

            // Storage writes
            // --------------
            // Add shipment (1 DB write)
            <Shipments<T>>::insert(&id, shipment);
            <ShipmentsOfOrganization<T>>::append(&owner, &id);

            Self::deposit_event(RawEvent::ShipmentRegistered(who, id.clone(), owner));
            Self::deposit_event(RawEvent::ShipmentStatusUpdated(id, status));

            Ok(())
        }

        #[weight = 10_000]
        pub fn record_event(origin, event: ShippingEvent<T::Moment>) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;

            // Validate extrinsic data (no storage access)
            // -----------------------
            // Validate format of event & shipment ID
            Self::validate_identifier(&event.id)?;
            Self::validate_identifier(&event.shipment_id)?;

            let event_id = event.id.clone();
            let event_type = event.event_type.clone();
            let shipment_id = event.shipment_id.clone();

            // Storage checks
            // --------------
            // Get event count (1 DB read)
            let event_count = EventCount::get();
            let event_idx = event_count.checked_add(1).ok_or(Error::<T>::ShippingEventMaxExceeded)?;
            // Check event doesn't exist yet (1 DB read)
            Self::validate_new_shipping_event(&event_id)?;

            // Check shipment is known (1 DB read)
            // Additionnally, we refuse some shipping events based on the shipment's status
            let mut shipment = match <Shipments<T>>::get(&shipment_id)
            {
                Some(shipment) => {
                    match shipment.status {
                        ShipmentStatus::Delivered => Err(<Error<T>>::ShipmentHasBeenDelivered),
                        ShipmentStatus::InTransit if event_type == ShippingEventType::ShipmentPickup => Err(<Error<T>>::ShipmentIsInTransit),
                        _ => Ok(shipment)
                    }
                }
                None => Err(<Error<T>>::ShipmentIsUnknown)
            }?;

            // Storage writes
            // --------------
            EventCount::put(event_idx);
            <AllEvents<T>>::insert(event_idx, event);
            EventIndices::insert(&event_id, event_idx);
            EventsOfShipment::append(&shipment_id, event_idx);

            Self::deposit_event(RawEvent::ShippingEventRecorded(who, event_id, shipment_id.clone(), event_type.clone()));

            match event_type {
                ShippingEventType::SensorReading => { /* Do nothing */ },
                _ => {
                    shipment = match event_type {
                        ShippingEventType::ShipmentPickup => shipment.pickup(),
                        ShippingEventType::ShipmentDelivery => shipment.deliver(<timestamp::Module<T>>::now()),
                        _ => unreachable!()
                    };
                    let new_status = shipment.status.clone();
                    <Shipments<T>>::insert(&shipment_id, shipment);
                    Self::deposit_event(RawEvent::ShipmentStatusUpdated(shipment_id, new_status));
                },
            }

            Ok(())
        }

	}
}

impl<T: Trait> AccountSet for Module<T> {
	type AccountId = T::AccountId;

	fn accounts() -> BTreeSet<T::AccountId> {
		<Members<T> as IterableStorageMap<T::AccountId, ()>>::iter()
			.map(|(acct, _)| acct)
			.collect::<BTreeSet<_>>()
	}
}

impl<T: Trait> Module<T> {
	// Helper methods
	fn new_shipment() -> ShipmentBuilder<T::AccountId, T::Moment> {
		ShipmentBuilder::<T::AccountId, T::Moment>::default()
	}

	pub fn validate_identifier(id: &[u8]) -> Result<(), Error<T>> {
		// Basic identifier validation
		ensure!(!id.is_empty(), Error::<T>::InvalidOrMissingIdentifier);
		ensure!(
            id.len() <= IDENTIFIER_MAX_LENGTH,
            Error::<T>::InvalidOrMissingIdentifier
        );
		Ok(())
	}

	pub fn validate_new_shipment(id: &[u8]) -> Result<(), Error<T>> {
		// Shipment existence check
		ensure!(
            !<Shipments<T>>::contains_key(id),
            Error::<T>::ShipmentAlreadyExists
        );
		Ok(())
	}

	pub fn validate_shipment_products(props: &[ProductId]) -> Result<(), Error<T>> {
		ensure!(
            props.len() <= SHIPMENT_MAX_PRODUCTS,
            Error::<T>::ShipmentHasTooManyProducts,
        );
		Ok(())
	}

	pub fn validate_new_shipping_event(id: &[u8]) -> Result<(), Error<T>> {
		// Shipping event existence check
		// let event_key = EventIndices::hashed_key_for(&event_id);
		ensure!(
            !EventIndices::contains_key(id),
            Error::<T>::ShippingEventAlreadyExists
        );
		Ok(())
	}
}

#[derive(Default)]
pub struct ShipmentBuilder<AccountId, Moment>
	where
		AccountId: Default,
		Moment: Default,
{
	id: ShipmentId,
	owner: AccountId,
	products: Vec<ProductId>,
	registered: Moment,
}

impl<AccountId, Moment> ShipmentBuilder<AccountId, Moment>
	where
		AccountId: Default,
		Moment: Default,
{
	pub fn identified_by(mut self, id: ShipmentId) -> Self {
		self.id = id;
		self
	}

	pub fn owned_by(mut self, owner: AccountId) -> Self {
		self.owner = owner;
		self
	}

	pub fn with_products(mut self, products: Vec<ProductId>) -> Self {
		self.products = products;
		self
	}

	pub fn registered_on(mut self, registered: Moment) -> Self {
		self.registered = registered;
		self
	}

	pub fn build(self) -> Shipment<AccountId, Moment> {
		Shipment::<AccountId, Moment> {
			id: self.id,
			owner: self.owner,
			products: self.products,
			registered: self.registered,
			status: ShipmentStatus::Pending,
			delivered: None,
		}
	}
}
