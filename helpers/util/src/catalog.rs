use frame_support::{
    codec::{Decode, Encode},
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult, ensure,
    sp_runtime::RuntimeDebug,
    // traits::EnsureOrigin,
};

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
