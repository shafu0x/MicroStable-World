use alloc::vec;
extern crate alloc;
use alloc::vec::Vec;
use crate::token::erc20;
use alloy_sol_types::sol;
use alloy_primitives::Address;
use stylus_sdk::{alloy_primitives::U256, prelude::*};

pub struct MicroParams;

impl erc20::Erc20Params for MicroParams {
    const NAME: &'static str = "Shafu USD";
    const SYMBOL: &'static str = "shUSD";
    const DECIMALS: u8 = 18;
}

sol_storage! {
    #[cfg_attr(any(feature = "sh-usd"), stylus_sdk::prelude::entrypoint)]
    pub struct ShUSD {
        #[borrow]
        erc20::Erc20<MicroParams> erc20;
        address manager;
    }
}

sol! {
    error OnlyManagerCanCall();
    error ERC20MintError();
    error ERC20BurnError();
}

#[derive(SolidityError)]
pub enum ShUSDErrors {
    OnlyManagerCanCall(OnlyManagerCanCall),
    ERC20MintErr(ERC20MintError),
    ERC20BurnErr(ERC20BurnError)
}

#[cfg_attr(feature = "sh-usd", stylus_sdk::prelude::public, inherit(erc20::Erc20::<MicroParams>))]
impl ShUSD {
    pub fn init(&mut self, manager_address: Address) {
        self.manager.set(manager_address);
    }

    pub fn mint(&mut self, to: Address, amount: U256) -> Result<(), ShUSDErrors> {
        if self.vm().msg_sender() != self.manager.get() {
            return Err(ShUSDErrors::OnlyManagerCanCall(OnlyManagerCanCall {}));
        }
        self.erc20
            .mint(to, amount)
            .map_err(|_| ShUSDErrors::ERC20MintErr(ERC20MintError{}))?;
        Ok(())
    }

    pub fn burn(&mut self, from: Address, amount: U256) -> Result<(), ShUSDErrors> {
        if self.vm().msg_sender() != self.manager.get() {
            return Err(ShUSDErrors::OnlyManagerCanCall(OnlyManagerCanCall {}));
        }
        self.erc20
            .burn(from, amount)
            .map_err(|_| ShUSDErrors::ERC20BurnErr(ERC20BurnError{}))?;
        Ok(())
    }
}
