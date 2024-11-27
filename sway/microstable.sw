contract;

use std::{
    auth::msg_sender,
    context::this_balance,
    bytes::Bytes,
    hash::Hash,
    storage::storage_string::*,
    string::String,
    context::msg_amount,
    call_frames::msg_asset_id,
    asset::{transfer, mint_to, burn},
    constants::DEFAULT_SUB_ID
};

use pyth_interface::{data_structures::price::{Price, PriceFeedId}, PythCore};

// Constants
pub const PYTH_CONTRACT_ID = 0x25146735b29d4216639f7f8b1d7b921ff87a1d3051de62d6cceaacabeb33b8e7;
pub const ETH_USD_PRICE_FEED  = 0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace;
pub const FUEL_ETH_BASE_ASSET_ID = 0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07;
pub const MIN_COLLAT_RATIO: u64 = 1_500_000_000_000_000_000; // 1.5e18

abi Manager {
    #[storage(read, write)]
    fn initialize();
    
    #[storage(read, write)]
    fn deposit(amount: u64);
    
    #[storage(read, write)]
    fn withdraw(amount: u64);
    
    #[storage(read, write)]
    fn mint(amount: u64);
    
    #[storage(read, write)]
    fn burn(amount: u64);
    
    #[storage(read, write)]
    fn liquidate(user: Identity);

    #[storage(read)]
    fn collat_ratio(user: Identity) -> u64;
}

storage {
    token_asset_id: AssetId = AssetId::from(b256::zero()),
    address_to_deposit: StorageMap<Identity, u64> = StorageMap {},
    address_to_minted: StorageMap<Identity, u64> = StorageMap {},
}

impl Manager for Contract {
    #[storage(read, write)]
    fn initialize() {
        let asset_id: AssetId = AssetId::default();
        storage.token_asset_id.write(asset_id);
    }

    #[storage(read, write)]
    fn deposit(amount: u64) {
        let sender = msg_sender().unwrap();
        // Check that the amount == msg_amount()
        require(msg_amount() != amount, "INSUFFICIENT_AMOUNT");

        require(msg_asset_id() == AssetId::from(FUEL_ETH_BASE_ASSET_ID), "INVALID_ASSET");
        let current_deposit = storage.address_to_deposit.get(sender).try_read().unwrap();
        storage.address_to_deposit.insert(
            sender,
            current_deposit + amount
        );
    }

    #[storage(read)]
    fn collat_ratio(user: Identity) -> u64 {
        let address_to_minted =  storage.address_to_minted.get(user).try_read().unwrap();
        let address_to_deposit =  storage.address_to_deposit.get(user).try_read().unwrap();
        _collat_ratio(user, address_to_minted, address_to_deposit)
    }

    #[storage(read, write)]
    fn withdraw(amount: u64) {
        let sender = msg_sender().unwrap();
        let current_deposit = storage.address_to_deposit.get(sender).try_read().unwrap();
        let current_minted = storage.address_to_minted.get(sender).try_read().unwrap();
        require(current_deposit >= amount, "INSUFFICIENT_DEPOSIT");
        
        storage.address_to_deposit.insert(sender, current_deposit - amount);
        
        require(_collat_ratio(Identity::from(sender), current_minted, current_deposit) >= MIN_COLLAT_RATIO,
            "COLLATERAL_RATIO_LESS_THAN_MIN"
        );
        
        transfer(sender, AssetId::from(FUEL_ETH_BASE_ASSET_ID), amount);
 
    }
    
    #[storage(read, write)]
    fn mint(amount: u64) {
        let sender = msg_sender().unwrap();
        let current_minted = storage.address_to_minted.get(sender).try_read().unwrap_or(0);
        let current_deposit = storage.address_to_deposit.get(sender).try_read().unwrap_or(0);
        storage.address_to_minted.insert(sender, current_minted + amount);
        require(
            _collat_ratio(sender, current_minted, current_deposit) >= MIN_COLLAT_RATIO, "CALLATERAL_RATIO_TOO_LOW"
        );

        mint_to(sender, DEFAULT_SUB_ID, amount);
    }
    
    #[storage(read, write)]
    fn burn(amount: u64) {
        require(msg_amount() == amount, "INSUFFICIENT_AMOUNT");
        require(msg_asset_id() == storage.token_asset_id.read(), "INVALID_ASSET");
        let sender = msg_sender().unwrap();
        let current_minted = storage.address_to_minted.get(sender).try_read().unwrap_or(0);

        require(current_minted >= amount, "INSUFFICIENT_BALANCE");
     
        storage.address_to_minted.insert(sender, current_minted - amount);
        burn(DEFAULT_SUB_ID, amount);
    }
    
    #[storage(read, write)]
    fn liquidate(user: Identity) {
        let current_minted = storage.address_to_minted.get(user).try_read().unwrap_or(0);
        let current_deposit = storage.address_to_deposit.get(user).try_read().unwrap_or(0);
        require(
            _collat_ratio(user, current_minted, current_deposit) < MIN_COLLAT_RATIO, "CANNOT_LIQUIDATE"
        );
        
        let sender = msg_sender().unwrap();
        
        require(msg_amount() == current_minted, "INSUFFICIENT_AMOUNT");
        require(msg_asset_id() == storage.token_asset_id.read(), "INVALID_ASSET");
        
        burn(DEFAULT_SUB_ID, current_minted);
        transfer(sender, AssetId::from(FUEL_ETH_BASE_ASSET_ID), current_deposit);

        
        storage.address_to_deposit.insert(user, 0);
        storage.address_to_minted.insert(user, 0);
    }
}

fn _collat_ratio(user: Identity, address_to_minted: u64, address_to_deposit: u64) -> u64 {
    let minted = address_to_minted;
    if minted == 0 {
        return u64::max();
    }
    
    let pyth_contract = abi(PythCore, PYTH_CONTRACT_ID);
    let price = pyth_contract.price_unsafe(ETH_USD_PRICE_FEED);
        
    let deposit = address_to_deposit;
    let total_value = (deposit * (price.price)) / MIN_COLLAT_RATIO;
    return total_value / minted;
}