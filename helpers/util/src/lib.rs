#![cfg_attr(not(feature = "std"), no_std)]

pub mod catalog;
pub mod shipment;

use sp_core::{sr25519, Pair, H256};

pub fn account_key(s: &str) -> sr25519::Public {
    sr25519::Pair::from_string(&format!("//{}", s), None)
        .expect("static values are valid; qed")
        .public()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
