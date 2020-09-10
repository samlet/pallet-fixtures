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