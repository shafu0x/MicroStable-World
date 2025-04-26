use alloc::vec;
extern crate alloc;
use alloc::vec::Vec;
use crate::token::erc20;
use alloy_sol_types::sol;
use alloy_primitives::Address;
use stylus_sdk::{alloy_primitives::U256, prelude::*};

pub struct MicroParams;

impl erc20::Erc20Params for MicroParams {
    const NAME: &'static str = "Wrapped Ether";
    const SYMBOL: &'static str = "WETH";
    const DECIMALS: u8 = 18;
}

sol_storage! {
    #[cfg_attr(any(feature = "test-weth"), stylus_sdk::prelude::entrypoint)]
    pub struct TestWeth {
        #[borrow]
        erc20::Erc20<MicroParams> erc20;
        address manager;
    }
}

sol! {
    error OnlyManagerCanCall();
}

#[derive(SolidityError)]
pub enum TestWethErrors {
    OnlyManagerCanCall(OnlyManagerCanCall),
}

#[cfg_attr(feature = "test-weth", stylus_sdk::prelude::public, inherit(erc20::Erc20::<MicroParams>))]
impl TestWeth {
    pub fn init(&mut self, manager_address: Address) {
        self.manager.set(manager_address);
    }

    pub fn mint(&mut self, to: Address, amount: U256) -> Result<(), TestWethErrors> {
        if self.vm().msg_sender() == self.manager.get() {
            let _ = self.erc20.mint(to, amount);
            Ok(())
        } else {
            Err(TestWethErrors::OnlyManagerCanCall(OnlyManagerCanCall {}))
        }
    }

    pub fn burn(&mut self, from: Address, amount: U256) -> Result<(), TestWethErrors> {
        if self.vm().msg_sender() == self.manager.get() {
            let _ = self.erc20.burn(from, amount);
            Ok(())
        } else {
            Err(TestWethErrors::OnlyManagerCanCall(OnlyManagerCanCall {}))
        }
    }
}

