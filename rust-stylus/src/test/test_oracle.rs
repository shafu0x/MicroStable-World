use alloc::vec;
use alloc::vec::Vec;
use stylus_sdk::storage::StorageBool;
use stylus_sdk::{alloy_primitives::I256, prelude::*};

const PRICE: I256 = I256::from_limbs([175_765_550_000, 0, 0, 0]);

#[cfg_attr(feature = "test-oracle", stylus_sdk::prelude::entrypoint)]
#[storage]
pub struct TestOracle {
    is_rekt: StorageBool
}

#[cfg_attr(feature = "test-oracle", stylus_sdk::prelude::public)]
#[cfg(feature = "test-oracle")]
impl TestOracle {
    pub fn latest_answer(&mut self) -> Result<I256, Vec<u8>> { 
        if self.is_rekt.get() {
            Ok(PRICE / I256::from_limbs([2, 0, 0, 0]))
        } else {
            Ok(PRICE)
        }
    }

    pub fn rekt(&mut self) {
        self.is_rekt.set(true)
    }
}
