use alloc::vec;
use alloc::vec::Vec;
use core::str::FromStr;
use alloy_sol_types::sol;
use crate::contracts::calls;
use alloy_primitives::Address;
use crate::alloc::string::ToString;
use stylus_sdk::{alloy_primitives::U256, prelude::*};
use stylus_sdk::storage::{StorageAddress, StorageMap, StorageU256, StorageBool};

const MIN_COLLAT_RATIO: u128 = 1_500_000_000_000_000_000; // 1.5e18

sol! {
    error Undercollateralized();
    error AlreadyInitialized();
    error CouldNotAdd();
    error CouldNotSub();
    error CouldNotMul();
    error CouldNotDiv();
    error ConversionFailure();
}

#[derive(SolidityError)]
pub enum ManagerErrors {
    Undercollateralized(Undercollateralized),
    AlreadyInitialized(AlreadyInitialized),
    CouldNotAdd(CouldNotAdd),
    CouldNotSub(CouldNotSub),
    CouldNotMul(CouldNotMul),
    CouldNotDiv(CouldNotDiv),
    ConversionFailure(ConversionFailure)
}

#[macro_export]
macro_rules! assert_or {
    ($cond:expr, $err:expr) => {
        if !($cond) {
            Err($err)?;
        }
    };
}

#[cfg_attr(feature = "manager", stylus_sdk::prelude::entrypoint)]
#[storage]
pub struct Manager {
    sh_usd: StorageAddress,
    weth: StorageAddress,
    oracle: StorageAddress,
    address_2deposit: StorageMap<Address, StorageU256>,
    address_2minted: StorageMap<Address, StorageU256>,
    is_initialized: StorageBool
}

#[cfg_attr(feature = "manager", stylus_sdk::prelude::public)]
#[cfg(feature = "manager")]
impl Manager {
    pub fn init(&mut self, weth_address: Address, oracle_address: Address, sh_usd_address: Address) -> Result<(), Vec<u8>> {
        assert_or!(!self.is_initialized.get(), ManagerErrors::AlreadyInitialized(AlreadyInitialized {}));
        self.weth.set(weth_address);
        self.oracle.set(oracle_address);
        self.sh_usd.set(sh_usd_address);
        self.is_initialized.set(true);
        Ok(())
    }

    pub fn deposit(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        let sender = self.vm().msg_sender();
        let this = self.vm().contract_address();
        calls::transfer_from_call(self.weth.get(), sender, this, amount)?;
        let previus_balance = self.address_2deposit.get(sender);
        self.address_2deposit.insert(sender, previus_balance.checked_add(amount)
            .ok_or(ManagerErrors::CouldNotAdd(CouldNotAdd {}))?);
        Ok(())
    }

    pub fn burn(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        let sender = self.vm().msg_sender();
        let previous_balance = self.address_2minted.get(sender);
        self.address_2minted.insert(sender, previous_balance.checked_sub(amount)
            .ok_or(ManagerErrors::CouldNotSub(CouldNotSub {}))?);
        calls::burn_call(self.sh_usd.get(), sender, amount)?;
        Ok(())
    }

    pub fn mint(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        let sender = self.vm().msg_sender();
        let previous_balance = self.address_2minted.get(sender);
        self.address_2minted.insert(sender, previous_balance.checked_add(amount)
            .ok_or(ManagerErrors::CouldNotAdd(CouldNotAdd {}))?);
        let ratio = self.collat_ratio(sender)?;
        assert_or!(ratio > U256::from(MIN_COLLAT_RATIO), ManagerErrors::Undercollateralized(Undercollateralized {}));
        calls::mint_call(self.sh_usd.get(), sender, amount)?;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        let sender = self.vm().msg_sender();
        let previous_deposit = self.address_2deposit.get(sender);
        self.address_2deposit.insert(sender, previous_deposit.checked_sub(amount)
            .ok_or(ManagerErrors::CouldNotSub(CouldNotSub {}))?);
        let ratio = self.collat_ratio(sender)?;
        assert_or!(ratio > U256::from(MIN_COLLAT_RATIO), ManagerErrors::Undercollateralized(Undercollateralized {}));
        calls::transfer_call(self.weth.get(), sender, amount)?;
        Ok(())
    }

    pub fn liquidate(&mut self, user: Address) -> Result<(), Vec<u8>> {
        let result = self.collat_ratio(user)?;
        assert_or!(result <= U256::from(MIN_COLLAT_RATIO), ManagerErrors::Undercollateralized(Undercollateralized {}));
        let sender = self.vm().msg_sender();
        let amount_minted = self.address_2minted.get(user);
        calls::burn_call(self.sh_usd.get(), user, amount_minted)?;
        let amount_deposited = self.address_2deposit.get(user);
        calls::transfer_call(self.weth.get(), sender, amount_deposited)?;
        self.address_2deposit.insert(user, U256::ZERO);
        self.address_2minted.insert(user, U256::ZERO);
        Ok(())
    }

    pub fn collat_ratio(&self, user: Address) -> Result<U256, Vec<u8>> {
        let minted = self.address_2minted.get(user);
        if minted.is_zero() { return Ok(U256::MAX); }
        let deposited = self.address_2deposit.get(user);
        let price = calls::latest_answer_call(self.oracle.get())?;
        let value = deposited.checked_mul(U256::from_str(&price.to_string()).unwrap()
            .checked_mul(U256::from(1e10 as u64))
            .ok_or(ManagerErrors::CouldNotMul(CouldNotMul {}))?)
        .ok_or(ManagerErrors::CouldNotMul(CouldNotMul {}))?;
        let value_scaled = value.checked_div(U256::from(1e18 as u64)).ok_or(ManagerErrors::CouldNotDiv(CouldNotDiv {}))?;
        Ok(value_scaled.checked_div(minted).ok_or(ManagerErrors::CouldNotDiv(CouldNotDiv {}))?)
    }
}
