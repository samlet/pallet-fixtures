use parity_scale_codec::{
    Decode,
    Encode,
};
// use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Relation<OrgId> {
    pub parent: OrgId,
    pub child: OrgId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        println!("just a test.");
    }
}