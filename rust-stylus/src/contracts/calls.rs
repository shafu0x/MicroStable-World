use alloc::vec::Vec;
use alloy_primitives::Address;
use stylus_sdk::{prelude::*, alloy_primitives::{I256, U256}, call::RawCall, alloy_sol_types::{sol, SolCall}};

sol! {
    error CouldNotCall();
    error CouldNotUnpackBool();
}

#[derive(SolidityError)]
pub enum CallErrors {
    CouldNotCall(CouldNotCall),
    CouldNotUnpackBool(CouldNotUnpackBool)
}

sol! {
    function latestAnswer() external view returns (int);
    function transferFrom(address from, address to, uint256 value) external returns (bool);
    function transfer(address to, uint256 value) external returns (bool);
    function burn(address from, uint256 amount) external;
    function mint(address to, uint256 amount) external;
}

pub fn latest_answer_call(oracle: Address) -> Result<I256, Vec<u8>> {
    I256::try_from_be_slice(unsafe { &RawCall::new().call(oracle, &latestAnswerCall {}.abi_encode()).unwrap()}).ok_or(CallErrors::CouldNotCall(CouldNotCall {}).into())
}

pub fn transfer_from_call(token: Address, from: Address, to: Address, value: U256) -> Result<(), Vec<u8>> {
    unpack_bool_safe(unsafe { &RawCall::new().call(token, &transferFromCall {
        from,
        to,
        value,
    }.abi_encode()).unwrap()})
}

pub fn transfer_call(token: Address, to: Address, value: U256) -> Result<(), Vec<u8>> {
    unpack_bool_safe(unsafe { &RawCall::new().call(token, &transferCall {
        to,
        value,
    }.abi_encode()).unwrap()})
}

pub fn mint_call(token: Address, to: Address, amount: U256) -> Result<(), Vec<u8>> {
    unpack_bool_safe(unsafe { &RawCall::new().call(token, &mintCall {
        to,
        amount,
    }.abi_encode()).unwrap()})
}

pub fn burn_call(token: Address, from: Address, amount: U256) -> Result<(), Vec<u8>> {
    unpack_bool_safe(unsafe { &RawCall::new().call(token, &burnCall {
        from,
        amount,
    }.abi_encode()).unwrap()})
}

pub fn unpack_bool_safe(data: &[u8]) -> Result<(), Vec<u8>> {
    match data.get(31) {
        None | Some(1) => Ok(()),
        _ => Err(CallErrors::CouldNotUnpackBool(CouldNotUnpackBool {}).into()),
    }
}