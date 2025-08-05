// use alloy::primitives::{fixed_bytes, B256, Log, Address, U256};
// pub struct Erc20Transfer {
//     pub transaction_hash: String,
//     pub block_number: u64,
//     pub from_address: String,
//     pub to_address: String,
//     pub amount: String,
//     pub token_address: String,
//     pub timestamp: u64,
// }

// pub const TRANSFER_EVENT_SIGNATURE: B256 = fixed_bytes!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");

// pub fn is_erc20_transfer(log: &Log) -> bool {
//     // log.topics().iter().any(|topic| topic == &TRANSFER_EVENT_SIGNATURE)
//     log.topics().first() == Some(&TRANSFER_EVENT_SIGNATURE)
// }

// fn parse_erc20_transfer(log: &Log, block_info: u64, tx_hash: String, timestamp: u64) -> Option<Erc20Transfer> {
//     let topics = log.topics();
//     if topics.len() < 3 || log.data.data.len() < 32 {
//         return None;
//     }
//     // let from_address = Address::from_slice(&log.topics()[1].as_slice()[12..]);
//     let from_address = Address::from_word(topics[1]);
//     // let to_address = Address::from_slice(&log.topics()[2].as_slice()[12..]);
//     let to_address = Address::from_word(topics[2]);
//     let amount = U256::from_be_slice(&log.data.data[0..32]);

//     Some(Erc20Transfer {
//         transaction_hash: tx_hash,
//         block_number: block_info,
//         from_address: format!("{:#x}", from_address),
//         to_address: format!("{:#x}", to_address),
//         amount: amount.to_string(),
//         // token_contract: format!("{:#x}", log.address),
//         timestamp
//     })
// }