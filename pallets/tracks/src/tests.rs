#![allow(dead_code)]
#![allow(unused_imports)]

use crate::*;
use frame_support::{assert_noop, assert_ok, impl_outer_event, impl_outer_origin, parameter_types};
use frame_system as system;
use sp_core::{sr25519, Pair, H256};
use sp_io::TestExternalities;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};
use fixed::types::U16F16;

use bluefn_util::catalog::ProductId;
use bluefn_util::shipment::*;
use bluefn_util::account_key;

impl_outer_origin! {
	pub enum Origin for TestRuntime {}
}

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TestRuntime;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl system::Trait for TestRuntime {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Index = u64;
	type Call = ();
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = sr25519::Public;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = TestEvent;
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type ModuleToIndex = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

mod vec_set {
	pub use crate::Event;
}

impl_outer_event! {
	pub enum TestEvent for TestRuntime {
		vec_set<T>,
		system<T>,
	}
}

parameter_types! {
	pub const MinimumPeriod: u64 = 1000;
}
impl timestamp::Trait for TestRuntime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

impl Trait for TestRuntime {
	type Event = TestEvent;
}

pub type Timestamp = timestamp::Module<TestRuntime>;
pub type System = system::Module<TestRuntime>;
pub type Tracks = Module<TestRuntime>;

pub struct ExtBuilder;

impl ExtBuilder {
	pub fn build() -> TestExternalities {
		let storage = system::GenesisConfig::default()
			.build_storage::<TestRuntime>()
			.unwrap();
		let mut ext = TestExternalities::from(storage);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

const TEST_SENDER: &str = "Alice";

#[test]
fn add_member_works() {
	ExtBuilder::build().execute_with(|| {
		let sender = account_key(TEST_SENDER);
		assert_ok!(Tracks::add_member(Origin::signed(sender)));

		let expected_event = TestEvent::vec_set(RawEvent::MemberAdded(sender));
		assert!(System::events().iter().any(|a| a.event == expected_event));

		assert!(<Members<TestRuntime>>::contains_key(sender));
	})
}


pub fn store_test_shipment<T: Trait>(
	id: ShipmentId,
	owner: T::AccountId,
	status: ShipmentStatus,
	products: Vec<ProductId>,
	registered: T::Moment,
) {
	Shipments::<T>::insert(
		id.clone(),
		Shipment {
			id,
			owner,
			status,
			products,
			registered,
			delivered: None,
		},
	);
}

pub fn store_test_event<T: Trait>(id: ShippingEventId, shipment_id: ShipmentId) {
	let event = ShippingEvent {
		id: id.clone(),
		event_type: ShippingEventType::ShipmentPickup,
		shipment_id: shipment_id.clone(),
		location: None,
		readings: vec![],
		timestamp: 42.into(),
	};
	let event_idx = EventCount::get().checked_add(1).unwrap();
	EventCount::put(event_idx);
	EventIndices::insert(id, event_idx);
	AllEvents::<T>::insert(event_idx, event);
	EventsOfShipment::append(shipment_id, event_idx);
}

const TEST_PRODUCT_ID: &str = "00012345678905";
const TEST_SHIPMENT_ID: &str = "0001";
const TEST_ORGANIZATION: &str = "Northwind";
const TEST_SHIPPING_EVENT_ID: &str = "9421fec019fb48299fbe";
const LONG_VALUE : &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec aliquam ut tortor nec congue. Pellente";

#[test]
fn register_shipment_without_products() {
	ExtBuilder::build().execute_with(|| {
		let sender = account_key(TEST_SENDER);
		let id = TEST_SHIPMENT_ID.as_bytes().to_owned();
		let owner = account_key(TEST_ORGANIZATION);
		let now = 42;
		Timestamp::set_timestamp(now);

		let result = Tracks::register_shipment(
			Origin::signed(sender),
			id.clone(),
			owner.clone(),
			vec![],
		);

		assert_ok!(result);

		assert_eq!(
			Tracks::shipment_by_id(&id),
			Some(Shipment {
				id: id.clone(),
				owner: owner,
				status: ShipmentStatus::Pending,
				products: vec![],
				registered: now,
				delivered: None
			})
		);

		assert_eq!(<ShipmentsOfOrganization<TestRuntime>>::get(owner), vec![id]);
	});
}


#[test]
fn register_shipment_with_invalid_sender() {
	ExtBuilder::build().execute_with(|| {
		assert_noop!(
            Tracks::register_shipment(
                Origin::none(),
                TEST_SHIPMENT_ID.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                vec!()
            ),
            dispatch::DispatchError::BadOrigin
        );
	});
}

#[test]
fn register_shipment_with_missing_id() {
	ExtBuilder::build().execute_with(|| {
		assert_noop!(
            Tracks::register_shipment(
                Origin::signed(account_key(TEST_SENDER)),
                vec!(),
                account_key(TEST_ORGANIZATION),
                vec!()
            ),
            Error::<TestRuntime>::InvalidOrMissingIdentifier
        );
	});
}

#[test]
fn register_shipment_with_long_id() {
	ExtBuilder::build().execute_with(|| {
		assert_noop!(
            Tracks::register_shipment(
                Origin::signed(account_key(TEST_SENDER)),
                LONG_VALUE.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                vec!()
            ),
            Error::<TestRuntime>::InvalidOrMissingIdentifier
        );
	})
}

#[test]
fn register_shipment_with_existing_id() {
	ExtBuilder::build().execute_with(|| {
		let existing_shipment = TEST_SHIPMENT_ID.as_bytes().to_owned();
		let now = 42;

		store_test_shipment::<TestRuntime>(
			existing_shipment.clone(),
			account_key(TEST_ORGANIZATION),
			ShipmentStatus::Pending,
			vec![],
			now,
		);

		assert_noop!(
            Tracks::register_shipment(
                Origin::signed(account_key(TEST_SENDER)),
                existing_shipment,
                account_key(TEST_ORGANIZATION),
                vec![]
            ),
            Error::<TestRuntime>::ShipmentAlreadyExists
        );
	})
}

#[test]
fn register_shipment_with_too_many_products() {
	ExtBuilder::build().execute_with(|| {
		assert_noop!(
            Tracks::register_shipment(
                Origin::signed(account_key(TEST_SENDER)),
                TEST_SHIPMENT_ID.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                vec![
                    b"00012345600001".to_vec(),
                    b"00012345600002".to_vec(),
                    b"00012345600003".to_vec(),
                    b"00012345600004".to_vec(),
                    b"00012345600005".to_vec(),
                    b"00012345600006".to_vec(),
                    b"00012345600007".to_vec(),
                    b"00012345600008".to_vec(),
                    b"00012345600009".to_vec(),
                    b"00012345600010".to_vec(),
                    b"00012345600011".to_vec(),
                ]
            ),
            Error::<TestRuntime>::ShipmentHasTooManyProducts
        );
	})
}

#[test]
fn record_event_with_invalid_sender() {
	ExtBuilder::build().execute_with(|| {
		let now = 42;

		assert_noop!(
            Tracks::record_event(
                Origin::none(),
                ShippingEvent {
                    id: TEST_SHIPPING_EVENT_ID.as_bytes().to_owned(),
                    event_type: ShippingEventType::ShipmentPickup,
                    shipment_id: TEST_SHIPMENT_ID.as_bytes().to_owned(),
                    location: None,
                    readings: vec![],
                    timestamp: now
                }
            ),
            dispatch::DispatchError::BadOrigin
        );
	});
}

#[test]
fn record_event_with_missing_event_id() {
	ExtBuilder::build().execute_with(|| {
		let now = 42;

		assert_noop!(
            Tracks::record_event(
                Origin::signed(account_key(TEST_SENDER)),
                ShippingEvent {
                    id: vec![],
                    event_type: ShippingEventType::ShipmentPickup,
                    shipment_id: TEST_SHIPMENT_ID.as_bytes().to_owned(),
                    location: None,
                    readings: vec![],
                    timestamp: now
                }
            ),
            Error::<TestRuntime>::InvalidOrMissingIdentifier,
        );
	});
}

#[test]
fn record_event_with_long_event_id() {
	ExtBuilder::build().execute_with(|| {
		let now = 42;

		assert_noop!(
            Tracks::record_event(
                Origin::signed(account_key(TEST_SENDER)),
                ShippingEvent {
                    id: LONG_VALUE.as_bytes().to_owned(),
                    event_type: ShippingEventType::ShipmentPickup,
                    shipment_id: TEST_SHIPMENT_ID.as_bytes().to_owned(),
                    location: None,
                    readings: vec![],
                    timestamp: now
                }
            ),
            Error::<TestRuntime>::InvalidOrMissingIdentifier,
        );
	});
}

#[test]
fn record_event_with_missing_shipment_id() {
	ExtBuilder::build().execute_with(|| {
		let now = 42;

		assert_noop!(
            Tracks::record_event(
                Origin::signed(account_key(TEST_SENDER)),
                ShippingEvent {
                    id: TEST_SHIPPING_EVENT_ID.as_bytes().to_owned(),
                    event_type: ShippingEventType::ShipmentPickup,
                    shipment_id: vec![],
                    location: None,
                    readings: vec![],
                    timestamp: now
                }
            ),
            Error::<TestRuntime>::InvalidOrMissingIdentifier
        );
	});
}

#[test]
fn record_event_with_long_shipment_id() {
	ExtBuilder::build().execute_with(|| {
		let now = 42;

		assert_noop!(
            Tracks::record_event(
                Origin::signed(account_key(TEST_SENDER)),
                ShippingEvent {
                    id: TEST_SHIPPING_EVENT_ID.as_bytes().to_owned(),
                    event_type: ShippingEventType::ShipmentPickup,
                    shipment_id: LONG_VALUE.as_bytes().to_owned(),
                    location: None,
                    readings: vec![],
                    timestamp: now
                }
            ),
            Error::<TestRuntime>::InvalidOrMissingIdentifier,
        );
	});
}

#[test]
fn record_event_with_existing_id() {
	ExtBuilder::build().execute_with(|| {
		let existing_event = hex::decode(TEST_SHIPPING_EVENT_ID).unwrap();
		let existing_shipment = TEST_SHIPMENT_ID.as_bytes().to_owned();
		let now = 42;

		store_test_shipment::<TestRuntime>(
			existing_shipment.clone(),
			account_key(TEST_ORGANIZATION),
			ShipmentStatus::Pending,
			vec![],
			now,
		);

		store_test_event::<TestRuntime>(existing_event.clone(), existing_shipment.clone());

		assert_noop!(
            Tracks::record_event(
                Origin::signed(account_key(TEST_SENDER)),
                ShippingEvent {
                    id: existing_event,
                    event_type: ShippingEventType::ShipmentPickup,
                    shipment_id: existing_shipment,
                    location: None,
                    readings: vec![],
                    timestamp: now
                }
            ),
            Error::<TestRuntime>::ShippingEventAlreadyExists,
        );
	})
}

#[test]
fn record_event_with_unknown_shipment() {
	ExtBuilder::build().execute_with(|| {
		let event_id = hex::decode(TEST_SHIPPING_EVENT_ID).unwrap();
		let unknown_shipment = TEST_SHIPMENT_ID.as_bytes().to_owned();
		let now = 42;

		assert_noop!(
            Tracks::record_event(
                Origin::signed(account_key(TEST_SENDER)),
                ShippingEvent {
                    id: event_id,
                    event_type: ShippingEventType::ShipmentPickup,
                    shipment_id: unknown_shipment,
                    location: None,
                    readings: vec![],
                    timestamp: now
                }
            ),
            Error::<TestRuntime>::ShipmentIsUnknown,
        );
	})
}

#[test]
fn record_event_for_shipment_pickup() {
	ExtBuilder::build().execute_with(|| {
		let owner = account_key(TEST_ORGANIZATION);
		let event_id = hex::decode(TEST_SHIPPING_EVENT_ID).unwrap();
		let shipment_id = TEST_SHIPMENT_ID.as_bytes().to_owned();
		let now = 42;

		store_test_shipment::<TestRuntime>(
			shipment_id.clone(),
			owner,
			ShipmentStatus::Pending,
			vec![TEST_PRODUCT_ID.as_bytes().to_owned()],
			now,
		);

		// Dispatchable call succeeds
		let event = ShippingEvent {
			id: event_id.clone(),
			event_type: ShippingEventType::ShipmentPickup,
			shipment_id: shipment_id.clone(),
			location: None,
			readings: vec![],
			timestamp: now,
		};
		assert_ok!(Tracks::record_event(
            Origin::signed(account_key(TEST_SENDER)),
            event.clone()
        ));

		// Storage is correctly updated
		assert_eq!(EventCount::get(), 1);
		assert_eq!(EventIndices::get(event_id), Some(1));
		assert_eq!(AllEvents::<TestRuntime>::get(1), Some(event));
		assert_eq!(EventsOfShipment::get(&shipment_id), vec![1]);

		// Shipment's status should be updated to 'InTransit'
		assert_eq!(
			Tracks::shipment_by_id(&shipment_id),
			Some(Shipment {
				id: shipment_id.clone(),
				owner: owner,
				status: ShipmentStatus::InTransit,
				products: vec![TEST_PRODUCT_ID.as_bytes().to_owned()],
				registered: now,
				delivered: None
			})
		);
	})
}

#[test]
fn record_event_for_shipment_delivery() {
	ExtBuilder::build().execute_with(|| {
		let owner = account_key(TEST_ORGANIZATION);
		let event_id = hex::decode(TEST_SHIPPING_EVENT_ID).unwrap();
		let shipment_id = TEST_SHIPMENT_ID.as_bytes().to_owned();
		let now = 42;
		Timestamp::set_timestamp(now);

		// Store shipment w/ InTransit status
		store_test_shipment::<TestRuntime>(
			shipment_id.clone(),
			owner,
			ShipmentStatus::InTransit,
			vec![TEST_PRODUCT_ID.as_bytes().to_owned()],
			now,
		);

		// Dispatchable call succeeds
		let event = ShippingEvent {
			id: event_id.clone(),
			event_type: ShippingEventType::ShipmentDelivery,
			shipment_id: shipment_id.clone(),
			location: None,
			readings: vec![],
			timestamp: now,
		};
		assert_ok!(Tracks::record_event(
            Origin::signed(account_key(TEST_SENDER)),
            event.clone()
        ));

		// Storage is correctly updated
		assert_eq!(EventCount::get(), 1);
		assert_eq!(EventIndices::get(event_id), Some(1));
		assert_eq!(AllEvents::<TestRuntime>::get(1), Some(event));
		assert_eq!(EventsOfShipment::get(&shipment_id), vec![1]);

		// Shipment's status should be updated to 'InTransit'
		// and delivered timestamp updated
		assert_eq!(
			Tracks::shipment_by_id(&shipment_id),
			Some(Shipment {
				id: shipment_id.clone(),
				owner: owner,
				status: ShipmentStatus::Delivered,
				products: vec![TEST_PRODUCT_ID.as_bytes().to_owned()],
				registered: now,
				delivered: Some(now)
			})
		);
	})
}

#[test]
fn record_event_for_sensor_reading() {
	ExtBuilder::build().execute_with(|| {
		let owner = account_key(TEST_ORGANIZATION);
		let event_id = hex::decode(TEST_SHIPPING_EVENT_ID).unwrap();
		let shipment_id = TEST_SHIPMENT_ID.as_bytes().to_owned();
		let now = 42;

		// Store shipment w/ InTransit status
		store_test_shipment::<TestRuntime>(
			shipment_id.clone(),
			owner,
			ShipmentStatus::InTransit,
			vec![TEST_PRODUCT_ID.as_bytes().to_owned()],
			now,
		);

		store_test_event::<TestRuntime>(
			hex::decode("88356e4576444cae8c78").unwrap(),
			shipment_id.clone(),
		);

		// Dispatchable call succeeds
		let event = ShippingEvent {
			id: event_id.clone(),
			event_type: ShippingEventType::SensorReading,
			shipment_id: shipment_id.clone(),
			location: Some(ReadPoint {
				latitude: U16F16::from_num(52.4941126),
				longitude: U16F16::from_num(13.4355606),
			}),
			readings: vec![Reading {
				device_id: "14d453ea4bdf46bc8042".as_bytes().to_owned(),
				reading_type: ReadingType::Temperature,
				value: U16F16::from_num(20.123),
				timestamp: now,
			}],
			timestamp: now,
		};
		assert_ok!(Tracks::record_event(
            Origin::signed(account_key(TEST_SENDER)),
            event.clone()
        ));

		// Storage is correctly updated
		assert_eq!(EventCount::get(), 2);
		assert_eq!(EventIndices::get(event_id), Some(2));
		assert_eq!(AllEvents::<TestRuntime>::get(2), Some(event));
		assert_eq!(EventsOfShipment::get(&shipment_id), vec![1, 2]);

		// Shipment's status should still be 'InTransit'
		assert_eq!(
			Tracks::shipment_by_id(&shipment_id),
			Some(Shipment {
				id: shipment_id.clone(),
				owner: owner,
				status: ShipmentStatus::InTransit,
				products: vec![TEST_PRODUCT_ID.as_bytes().to_owned()],
				registered: now,
				delivered: None
			})
		);
	})
}

#[test]
fn record_event_for_delivered_shipment() {
	ExtBuilder::build().execute_with(|| {
		let owner = account_key(TEST_ORGANIZATION);
		let event_id = hex::decode(TEST_SHIPPING_EVENT_ID).unwrap();
		let shipment_id = TEST_SHIPMENT_ID.as_bytes().to_owned();
		let now = 42;

		// Store shipment w/ Delivered status
		store_test_shipment::<TestRuntime>(
			shipment_id.clone(),
			owner,
			ShipmentStatus::Delivered,
			vec![TEST_PRODUCT_ID.as_bytes().to_owned()],
			now,
		);

		assert_noop!(
            Tracks::record_event(
                Origin::signed(account_key(TEST_SENDER)),
                ShippingEvent {
                    id: event_id.clone(),
                    event_type: ShippingEventType::ShipmentPickup,
                    shipment_id: shipment_id.clone(),
                    location: None,
                    readings: vec![],
                    timestamp: now,
                }
            ),
            Error::<TestRuntime>::ShipmentHasBeenDelivered
        );
	})
}

#[test]
fn record_event_for_intransit_shipment() {
	ExtBuilder::build().execute_with(|| {
		let owner = account_key(TEST_ORGANIZATION);
		let event_id = hex::decode(TEST_SHIPPING_EVENT_ID).unwrap();
		let shipment_id = TEST_SHIPMENT_ID.as_bytes().to_owned();
		let now = 42;

		// Store shipment w/ InTransit status
		store_test_shipment::<TestRuntime>(
			shipment_id.clone(),
			owner,
			ShipmentStatus::InTransit,
			vec![TEST_PRODUCT_ID.as_bytes().to_owned()],
			now,
		);

		assert_noop!(
            Tracks::record_event(
                Origin::signed(account_key(TEST_SENDER)),
                ShippingEvent {
                    id: event_id.clone(),
                    event_type: ShippingEventType::ShipmentPickup,
                    shipment_id: shipment_id.clone(),
                    location: None,
                    readings: vec![],
                    timestamp: now,
                }
            ),
            Error::<TestRuntime>::ShipmentIsInTransit
        );
	})
}
