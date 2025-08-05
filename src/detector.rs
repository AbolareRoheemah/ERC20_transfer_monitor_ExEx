use std::collections::HashMap;
use alloy::{primitives::{fixed_bytes, Address, FixedBytes, Log, B256, U256}};
use crate::transfer::{TokenInfo, TransferEvent, TransferFilter};

pub struct TransferDetector {
    filter: TransferFilter,
    known_tokens: HashMap<Address, TokenInfo>,
}

const TRANSFER_EVENT_SIGNATURE: B256 = fixed_bytes!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");
const USDC: FixedBytes<20> = fixed_bytes!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
const DAI: FixedBytes<20> = fixed_bytes!("6b175474e89094c44da98b954eedeac495271d0f");
const WETH: FixedBytes<20> = fixed_bytes!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");

impl TransferDetector {
    pub fn new(filter: TransferFilter) -> Self {
        Self {
            filter,
            known_tokens: HashMap::new(),
        }
    }

    fn add_known_token(&mut self, address: Address, info: TokenInfo) {
        self.known_tokens.insert(address, info);
    }

    pub fn is_transfer_log(log: &Log) -> bool {
        log.topics().first() == Some(&TRANSFER_EVENT_SIGNATURE)
    }

    pub fn parse_transfer_log(log: &Log, block_number: u64, tx_hash: B256) -> Option<TransferEvent> {
        let topics = log.topics();
        if topics.len() < 3 || log.data.data.len() < 32 {
            return None;
        }
        let from = Address::from_word(topics[1]);
        let to = Address::from_word(topics[2]);
        let value = U256::from_be_slice(&log.data.data[0..32]);

        Some(TransferEvent {
            token_address: log.address,
            from,
            to,
            value,
            block_number,
            transaction_hash: tx_hash,
            log_index: 0
        })
    }

    fn decode_address_from_topic(log: &Log) -> Address {
        let topics = log.topics();
        Address::from_word(topics[1])
    }

    fn decode_value_from_data(log: &Log) -> U256 {
        U256::from_be_slice(&log.data.data[0..32])
    }
}