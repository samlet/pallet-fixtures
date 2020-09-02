//! A pallet to demonstrate usage of a simple storage map
//!
//! Storage maps map a key type to a value type. The hasher used to hash the key can be customized.
//! This pallet uses the `blake2_128_concat` hasher. This is a good default hasher.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	codec::{Decode, Encode},
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::DispatchResult, ensure,
	sp_runtime::RuntimeDebug,
	// traits::EnsureOrigin,
};
use frame_system::{self as system, ensure_signed};

#[cfg(test)]
mod tests;

//++
// General constraints to limit data size
// Note: these could also be passed as trait config parameters
pub const PRODUCT_ID_MAX_LENGTH: usize = 14;
pub const PRODUCT_PROP_NAME_MAX_LENGTH: usize = 10;
pub const PRODUCT_PROP_VALUE_MAX_LENGTH: usize = 20;
pub const PRODUCT_MAX_PROPS: usize = 3;

// Custom types
pub type ProductId = Vec<u8>;
pub type PropName = Vec<u8>;
pub type PropValue = Vec<u8>;

// Product contains master data (aka class-level) about a trade item.
// This data is typically registered once by the product's manufacturer / supplier,
// to be shared with other network participants, and remains largely static.
// It can also be used for instance-level (lot) master data.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Product<AccountId, Moment> {
	// The product ID would typically be a GS1 GTIN (Global Trade Item Number),
	// or ASIN (Amazon Standard Identification Number), or similar,
	// a numeric or alpha-numeric code with a well-defined data structure.
	id: ProductId,
	// This is account that represents the owner of this product, as in
	// the manufacturer or supplier providing this product within the value chain.
	owner: AccountId,
	// This a series of properties describing the product.
	// Typically, there would at least be a textual description, and SKU.
	// It could also contain instance / lot master data e.g. expiration, weight, harvest date.
	props: Option<Vec<ProductProperty>>,
	// Timestamp (approximate) at which the prodct was registered on-chain.
	registered: Moment,
}

// Contains a name-value pair for a product property e.g. description: Ingredient ABC
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ProductProperty {
	// Name of the product property e.g. desc or description
	name: PropName,
	// Value of the product property e.g. Ingredient ABC
	value: PropValue,
}

impl ProductProperty {
	pub fn new(name: &[u8], value: &[u8]) -> Self {
		Self {
			name: name.to_vec(),
			value: value.to_vec(),
		}
	}

	pub fn name(&self) -> &[u8] {
		self.name.as_ref()
	}

	pub fn value(&self) -> &[u8] {
		self.value.as_ref()
	}
}
//++

pub trait Trait: system::Trait + timestamp::Trait  {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	// type CreateRoleOrigin: EnsureOrigin<Self::Origin>;
}

decl_storage! {
	trait Store for Module<T: Trait> as SimpleMap {
		SimpleMap get(fn simple_map): map hasher(blake2_128_concat) T::AccountId => u32;

		/// Product
		pub Products get(fn product_by_id): map hasher(blake2_128_concat) ProductId => Option<Product<T::AccountId, T::Moment>>;
        pub ProductsOfOrganization get(fn products_of_org): map hasher(blake2_128_concat) T::AccountId => Vec<ProductId>;
        pub OwnerOf get(fn owner_of): map hasher(blake2_128_concat) ProductId => Option<T::AccountId>;
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		/// A user has set their enrty
		EntrySet(AccountId, u32),

		/// A user has read their entry, leaving it in storage
		EntryGot(AccountId, u32),

		/// A user has read their entry removing it fro mstorage
		EntryTaken(AccountId, u32),

		/// A user has read their entry, incremented it, and writtenthe new entry to storage
		/// Parameters are (user, old_entry, new_entry)
		EntryIncreased(AccountId, u32, u32),

		/// Products
		ProductRegistered(AccountId, ProductId, AccountId),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// The requested user has not stored a value yet
		NoValueStored,

		/// The value cannot be incremented further because it has reached the maimum allowed value
		MaxValueReached,

		/// Products
		ProductIdMissing,
        ProductIdTooLong,
        ProductIdExists,
        ProductTooManyProps,
        ProductInvalidPropName,
        ProductInvalidPropValue,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		// Initialize errors
		type Error = Error<T>;

		// Initialize events
		fn deposit_event() = default;

		/// Set the value stored at a particular key
		#[weight = 10_000]
		fn set_single_entry(origin, entry: u32) -> DispatchResult {
			// A user can only set their own entry
			let user = ensure_signed(origin)?;

			<SimpleMap<T>>::insert(&user, entry);

			Self::deposit_event(RawEvent::EntrySet(user, entry));
			Ok(())
		}

		/// Read the value stored at a particular key and emit it in an event
		#[weight = 10_000]
		fn get_single_entry(origin, account: T::AccountId) -> DispatchResult {
			// Any user can get any other user's entry
			let getter = ensure_signed(origin)?;

			ensure!(<SimpleMap<T>>::contains_key(&account), Error::<T>::NoValueStored);
			let entry = <SimpleMap<T>>::get(account);
			Self::deposit_event(RawEvent::EntryGot(getter, entry));
			Ok(())
		}

		/// Read the value stored at a particular key,while removing it from the map.
		/// Also emit the read value in an event
		#[weight = 10_000]
		fn take_single_entry(origin) -> DispatchResult {
			// A user can only take (delete) their own entry
			let user = ensure_signed(origin)?;

			ensure!(<SimpleMap<T>>::contains_key(&user), Error::<T>::NoValueStored);
			let entry = <SimpleMap<T>>::take(&user);
			Self::deposit_event(RawEvent::EntryTaken(user, entry));
			Ok(())
		}

		/// Increase the value associated with a particular key
		#[weight = 10_000]
		fn increase_single_entry(origin, add_this_val: u32) -> DispatchResult {
			// A user can only mutate their own entry
			let user = ensure_signed(origin)?;

			ensure!(<SimpleMap<T>>::contains_key(&user), Error::<T>::NoValueStored);
			let original_value = <SimpleMap<T>>::get(&user);

			let new_value = original_value.checked_add(add_this_val).ok_or(Error::<T>::MaxValueReached)?;
			<SimpleMap<T>>::insert(&user, new_value);

			Self::deposit_event(RawEvent::EntryIncreased(user, original_value, new_value));

			Ok(())
		}

		// --- Products ---
		#[weight = 10_000]
        pub fn register_product(origin, id: ProductId, owner: T::AccountId, props: Option<Vec<ProductProperty>>) -> DispatchResult {
            // T::CreateRoleOrigin::ensure_origin(origin.clone())?;
            let who = ensure_signed(origin)?;

            // Validate product ID
            Self::validate_product_id(&id)?;

            // Validate product props
            Self::validate_product_props(&props)?;

            // Check product doesn't exist yet (1 DB read)
            Self::validate_new_product(&id)?;

            // TODO: if organization has an attribute w/ GS1 Company prefix,
            //       additional validation could be applied to the product ID
            //       to ensure its validity (same company prefix as org).

            // Create a product instance
            let product = Self::new_product()
                .identified_by(id.clone())
                .owned_by(owner.clone())
                .registered_on(<timestamp::Module<T>>::now())
                .with_props(props)
                .build();

            // Add product & ownerOf (3 DB writes)
            <Products<T>>::insert(&id, product);
            <ProductsOfOrganization<T>>::append(&owner, &id);
            <OwnerOf<T>>::insert(&id, &owner);

            Self::deposit_event(RawEvent::ProductRegistered(who, id, owner));

            Ok(())
        }
	}
}

//+ products
impl<T: Trait> Module<T> {
	// Helper methods
	fn new_product() -> ProductBuilder<T::AccountId, T::Moment> {
		ProductBuilder::<T::AccountId, T::Moment>::default()
	}

	pub fn validate_product_id(id: &[u8]) -> Result<(), Error<T>> {
		// Basic product ID validation
		ensure!(!id.is_empty(), Error::<T>::ProductIdMissing);
		ensure!(
            id.len() <= PRODUCT_ID_MAX_LENGTH,
            Error::<T>::ProductIdTooLong
        );
		Ok(())
	}

	pub fn validate_new_product(id: &[u8]) -> Result<(), Error<T>> {
		// Product existence check
		ensure!(
            !<Products<T>>::contains_key(id),
            Error::<T>::ProductIdExists
        );
		Ok(())
	}

	pub fn validate_product_props(props: &Option<Vec<ProductProperty>>) -> Result<(), Error<T>> {
		if let Some(props) = props {
			ensure!(
                props.len() <= PRODUCT_MAX_PROPS,
                Error::<T>::ProductTooManyProps,
            );
			for prop in props {
				ensure!(
                    prop.name().len() <= PRODUCT_PROP_NAME_MAX_LENGTH,
                    Error::<T>::ProductInvalidPropName
                );
				ensure!(
                    prop.value().len() <= PRODUCT_PROP_VALUE_MAX_LENGTH,
                    Error::<T>::ProductInvalidPropValue
                );
			}
		}
		Ok(())
	}
}

//+ helpers
#[derive(Default)]
pub struct ProductBuilder<AccountId, Moment>
	where
		AccountId: Default,
		Moment: Default,
{
	id: ProductId,
	owner: AccountId,
	props: Option<Vec<ProductProperty>>,
	registered: Moment,
}

impl<AccountId, Moment> ProductBuilder<AccountId, Moment>
	where
		AccountId: Default,
		Moment: Default,
{
	pub fn identified_by(mut self, id: ProductId) -> Self {
		self.id = id;
		self
	}

	pub fn owned_by(mut self, owner: AccountId) -> Self {
		self.owner = owner;
		self
	}

	pub fn with_props(mut self, props: Option<Vec<ProductProperty>>) -> Self {
		self.props = props;
		self
	}

	pub fn registered_on(mut self, registered: Moment) -> Self {
		self.registered = registered;
		self
	}

	pub fn build(self) -> Product<AccountId, Moment> {
		Product::<AccountId, Moment> {
			id: self.id,
			owner: self.owner,
			props: self.props,
			registered: self.registered,
		}
	}
}
