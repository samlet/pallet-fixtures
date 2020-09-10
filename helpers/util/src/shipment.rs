use frame_support::{
    codec::{Decode, Encode},
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult, ensure,
    sp_runtime::RuntimeDebug,
    // traits::EnsureOrigin,
};
use fixed::types::U16F16;
use crate::catalog::ProductId;

// General constraints to limit data size
// Note: these could also be passed as trait config parameters
pub const IDENTIFIER_MAX_LENGTH: usize = 10;
pub const SHIPMENT_MAX_PRODUCTS: usize = 10;

// Custom types
pub type Identifier = Vec<u8>;
pub type Decimal = U16F16;
pub type ShipmentId = Identifier;
pub type ShippingEventId = Identifier;
pub type ShippingEventIndex = u64;
pub type DeviceId = Identifier;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum ShipmentStatus {
    Pending,
    InTransit,
    Delivered,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Shipment<AccountId, Moment> {
    pub id: ShipmentId,
    pub owner: AccountId,
    pub status: ShipmentStatus,
    pub products: Vec<ProductId>,
    pub registered: Moment,
    pub delivered: Option<Moment>,
}

impl<AccountId, Moment> Shipment<AccountId, Moment> {
    pub fn pickup(mut self) -> Self {
        self.status = ShipmentStatus::InTransit;
        self
    }

    pub fn deliver(mut self, delivered_on: Moment) -> Self {
        self.status = ShipmentStatus::Delivered;
        self.delivered = Some(delivered_on);
        self
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum ShippingEventType {
    ShipmentPickup,
    SensorReading,
    ShipmentDelivery,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ShippingEvent<Moment> {
    pub id: ShippingEventId,
    pub event_type: ShippingEventType,
    pub shipment_id: ShipmentId,
    pub location: Option<ReadPoint>,
    pub readings: Vec<Reading<Moment>>,
    pub timestamp: Moment,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ReadPoint {
    pub latitude: Decimal,
    pub longitude: Decimal,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum ReadingType {
    Humidity,
    Pressure,
    Shock,
    Tilt,
    Temperature,
    Vibration,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Reading<Moment> {
    pub device_id: DeviceId,
    pub reading_type: ReadingType,
    pub timestamp: Moment,
    pub value: Decimal,
}

