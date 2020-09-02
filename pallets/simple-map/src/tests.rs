// use super::{RawEvent, Product, ProductId, Products, ProductsOfOrganization};
use super::*;
use crate::{Error, Module, Trait};
use frame_support::{assert_err, assert_ok, impl_outer_event, impl_outer_origin, parameter_types};
use frame_system as system;
use sp_core::{sr25519, Pair, H256};
use sp_io::TestExternalities;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};
use core::marker::PhantomData;

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
	type AccountId = u64;
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

mod simple_map {
	pub use crate::Event;
}
// use crate as product_registry;

impl_outer_origin! {
    pub enum Origin for TestRuntime {}
}
impl_outer_event! {
	pub enum TestEvent for TestRuntime {
		simple_map<T>,
		system<T>,
		// product_registry<T>,
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

pub struct MockOrigin<T>(PhantomData<T>);
impl Trait for TestRuntime {
	type Event = TestEvent;
	// type CreateRoleOrigin = MockOrigin<TestRuntime>;
}

pub type Timestamp = timestamp::Module<TestRuntime>;
pub type System = system::Module<TestRuntime>;
pub type SimpleMap = Module<TestRuntime>;

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

#[test]
fn set_works() {
	ExtBuilder::build().execute_with(|| {
		assert_ok!(SimpleMap::set_single_entry(Origin::signed(1), 19));

		let expected_event = TestEvent::simple_map(RawEvent::EntrySet(1, 19));

		assert!(System::events().iter().any(|a| a.event == expected_event));
	})
}

#[test]
fn get_throws() {
	ExtBuilder::build().execute_with(|| {
		assert_err!(
			SimpleMap::get_single_entry(Origin::signed(2), 3),
			Error::<TestRuntime>::NoValueStored
		);
	})
}

#[test]
fn get_works() {
	ExtBuilder::build().execute_with(|| {
		assert_ok!(SimpleMap::set_single_entry(Origin::signed(2), 19));
		assert_ok!(SimpleMap::get_single_entry(Origin::signed(1), 2));

		let expected_event = TestEvent::simple_map(RawEvent::EntryGot(1, 19));
		assert!(System::events().iter().any(|a| a.event == expected_event));

		// Ensure storage is still set
		assert_eq!(SimpleMap::simple_map(2), 19);
	})
}

#[test]
fn take_throws() {
	ExtBuilder::build().execute_with(|| {
		assert_err!(
			SimpleMap::take_single_entry(Origin::signed(2)),
			Error::<TestRuntime>::NoValueStored
		);
	})
}

#[test]
fn take_works() {
	ExtBuilder::build().execute_with(|| {
		assert_ok!(SimpleMap::set_single_entry(Origin::signed(2), 19));
		assert_ok!(SimpleMap::take_single_entry(Origin::signed(2)));

		let expected_event = TestEvent::simple_map(RawEvent::EntryTaken(2, 19));
		assert!(System::events().iter().any(|a| a.event == expected_event));

		// Assert storage has returned to default value (zero)
		assert_eq!(SimpleMap::simple_map(2), 0);
	})
}

#[test]
fn increase_works() {
	ExtBuilder::build().execute_with(|| {
		assert_ok!(SimpleMap::set_single_entry(Origin::signed(2), 19));
		assert_ok!(SimpleMap::increase_single_entry(Origin::signed(2), 2));

		let expected_event = TestEvent::simple_map(RawEvent::EntryIncreased(2, 19, 21));

		assert!(System::events().iter().any(|a| a.event == expected_event));
	})
}

// ++ Products
pub fn store_test_product<T: Trait>(id: ProductId, owner: T::AccountId, registered: T::Moment) {
	Products::<T>::insert(
		id.clone(),
		Product {
			id,
			owner,
			registered,
			props: None,
		},
	);
}

pub fn account_key(s: &str) -> sr25519::Public {
	sr25519::Pair::from_string(&format!("//{}", s), None)
		.expect("static values are valid; qed")
		.public()
}


const TEST_PRODUCT_ID: &str = "00012345600012";
const TEST_ORGANIZATION: &str = "Northwind";
const TEST_SENDER: &str = "Alice";
const LONG_VALUE : &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec aliquam ut tortor nec congue. Pellente";

#[test]
fn create_product_without_props() {
	ExtBuilder::build().execute_with(|| {
		// let sender = account_key(TEST_SENDER);
		let sender=Origin::signed(1);
		let id = TEST_PRODUCT_ID.as_bytes().to_owned();
		// let owner = account_key(TEST_ORGANIZATION);
		let owner=2;
		let now = 42;
		Timestamp::set_timestamp(now);

		let result = SimpleMap::register_product(
			// Origin::signed(sender),
			sender.clone(),
			id.clone(),
			owner.clone(),
			None,
		);

		assert_ok!(result);

		assert_eq!(
			SimpleMap::product_by_id(&id),
			Some(Product {
				id: id.clone(),
				owner: owner,
				registered: now,
				props: None
			})
		);

		assert_eq!(<ProductsOfOrganization<TestRuntime>>::get(owner), vec![id.clone()]);

		assert_eq!(SimpleMap::owner_of(&id), Some(owner));

		// Event is raised
		assert!(System::events().iter().any(|er| er.event
			== TestEvent::simple_map(RawEvent::ProductRegistered(
			1,
			id.clone(),
			owner
		))));
	});
}