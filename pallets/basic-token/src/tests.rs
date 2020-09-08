use crate::{Error, Module, Trait};
use frame_support::{assert_noop, assert_ok, impl_outer_origin, parameter_types, impl_outer_event};
use frame_system as system;
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

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

mod basic_token {
	pub use crate::Event;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 1000;
}

impl_outer_event! {
	pub enum TestEvent for TestRuntime {
		vec_set<T>,
		system<T>,
		basic_token<T>,
	}
}

impl timestamp::Trait for TestRuntime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}
impl vec_set::Trait for TestRuntime {
	type Event = TestEvent;
}

impl Trait for TestRuntime {
	type Event = TestEvent;
}


pub type BasicToken = Module<TestRuntime>;
pub type Timestamp = timestamp::Module<TestRuntime>;
pub type VecSet = vec_set::Module<TestRuntime>;
pub type System = system::Module<TestRuntime>;

pub struct ExtBuilder;

impl ExtBuilder {
	pub fn build() -> TestExternalities {
		let storage = system::GenesisConfig::default()
			.build_storage::<TestRuntime>()
			.unwrap();
		TestExternalities::from(storage)
	}
}

#[test]
fn init_works() {
	ExtBuilder::build().execute_with(|| {
		assert_ok!(BasicToken::init(Origin::signed(1)));
		assert_eq!(BasicToken::get_balance(1), 21000000);
	})
}

#[test]
fn cant_double_init() {
	ExtBuilder::build().execute_with(|| {
		assert_ok!(BasicToken::init(Origin::signed(1)));
		assert_noop!(
			BasicToken::init(Origin::signed(1)),
			Error::<TestRuntime>::AlreadyInitialized
		);
	})
}

#[test]
fn transfer_works() {
	ExtBuilder::build().execute_with(|| {
		assert_ok!(BasicToken::init(Origin::signed(1)));

		// Transfer 100 tokens from user 1 to user 2
		assert_ok!(BasicToken::transfer(Origin::signed(1), 2, 100));

		assert_eq!(BasicToken::get_balance(1), 20999900);
		assert_eq!(BasicToken::get_balance(2), 100);
	})
}

#[test]
fn cant_spend_more_than_you_have() {
	ExtBuilder::build().execute_with(|| {
		assert_ok!(BasicToken::init(Origin::signed(1)));
		assert_noop!(
			BasicToken::transfer(Origin::signed(1), 2, 21000001),
			Error::<TestRuntime>::InsufficientFunds
		);
	})
}

#[test]
fn test_timestamp() {
	ExtBuilder::build().execute_with(|| {
		let now = 42;
		Timestamp::set_timestamp(now);
		println!("{:?}", Timestamp::get());
	})
}

#[test]
fn test_vec_set() {
	ExtBuilder::build().execute_with(|| {
		assert_ok!(VecSet::add_member(Origin::signed(1)));

		// let expected_event = TestEvent::vec_set(RawEvent::MemberAdded(1));
		// assert!(System::events().iter().any(|a| a.event == expected_event));

		assert_eq!(VecSet::members(), vec![1]);
	})
}

#[test]
fn members_can_call() {
	ExtBuilder::build().execute_with(|| {
		assert_ok!(VecSet::add_member(Origin::signed(1)));

		// assert_ok!(CheckMembership::check_membership(Origin::signed(1)));
		// let expected_event = TestEvent::check_membership(RawEvent::IsAMember(1));
		// assert!(System::events().iter().any(|a| a.event == expected_event));
	})
}
