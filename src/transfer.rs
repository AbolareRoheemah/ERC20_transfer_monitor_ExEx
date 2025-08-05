use alloy::primitives::{Address, B256, U256};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferEvent {
    pub token_address: Address,
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub block_number: u64,
    pub transaction_hash: B256,
    pub log_index: u64
}

pub struct TokenInfo {
    symbol: String,
    decimals: u8,
    name: String
}

#[derive(Debug, Clone)]
pub enum TransferFilter {
    All,
    SpecificTokens(Vec<Address>),
    LargeTransfers(U256),
    SpecificAddresses(Vec<Address>)
}