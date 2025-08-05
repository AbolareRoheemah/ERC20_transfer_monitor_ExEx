use alloy::{primitives::{U256, Address, fixed_bytes}};
use reth::{api::{FullNodeComponents, NodeTypes}, primitives::EthPrimitives, providers::Chain};
use reth_exex::{ExExContext, ExExEvent, ExExNotification};
use futures_util::TryStreamExt;
use reth_node_ethereum::EthereumNode;
use reth_tracing::tracing::info;
use crate::{detector::TransferDetector, transfer::{TransferFilter, TransferEvent}};
use std::{error::Error, hash::Hash, collections::HashMap};

mod event;
mod database;
mod exex;
mod transfer;
mod detector;

/// Example filter: only log large transfers and transfers involving specific addresses/tokens.
/// In a real app, you might want to make this configurable.
fn apply_transfer_filter(transfer: &TransferEvent, filter: &TransferFilter) -> bool {
    match filter {
        TransferFilter::All => true,
        TransferFilter::LargeTransfers(threshold) => &transfer.value >= threshold,
        TransferFilter::SpecificTokens(tokens) => tokens.contains(&transfer.token_address),
        TransferFilter::SpecificAddresses(addresses) => {
            addresses.contains(&transfer.from) || addresses.contains(&transfer.to)
        }
    }
}

/// Helper: format an Ethereum address as 0x123456...abcd
fn format_address(addr: &Address) -> String {
    let hex = format!("{:x}", addr);
    let hex = if hex.len() < 40 {
        // pad with zeros if needed
        format!("{:0>40}", hex)
    } else {
        hex
    };
    format!("0x{}...{}", &hex[..6], &hex[36..])
}

/// Helper: convert U256 value to human-readable string using decimals
fn format_amount(value: &U256, decimals: u8) -> String {
    // Convert to string with decimal point at the right place
    let ten = U256::from(10u64);
    let divisor = ten.pow(U256::from(decimals));
    let int_part = value / divisor;
    let frac_part = value % divisor;

    if decimals == 0 {
        return format!("{}", int_part);
    }
    
    let mut frac_str = frac_part.to_string();
    let pad = decimals as usize - frac_str.len();
    if pad > 0 {
        frac_str = "0".repeat(pad) + &frac_str;
    }
    // Remove trailing zeros for aesthetics
    let frac_str = frac_str.trim_end_matches('0');
    if frac_str.is_empty() {
        format!("{}", int_part)
    } else {
        format!("{}.{}", int_part, frac_str)
    }
}

/// Known tokens for symbol/decimals lookup
fn known_tokens() -> HashMap<Address, (&'static str, u8)> {
    let mut map = HashMap::new();
    // USDC
    map.insert(Address::from(fixed_bytes!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")), ("USDC", 6));
    // DAI
    map.insert(Address::from(fixed_bytes!("6b175474e89094c44da98b954eedeac495271d0f")), ("DAI", 18));
    // WETH
    map.insert(Address::from(fixed_bytes!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2")), ("WETH", 18));
    map
}

/// Format a transfer event as a human-readable string
fn format_transfer(transfer: &TransferEvent) -> String {
    let tokens = known_tokens();
    let (symbol, decimals) = tokens
        .get(&transfer.token_address)
        .cloned()
        .unwrap_or(("UNKNOWN", 18)); // Default to 18 decimals if unknown

    let amount = format_amount(&transfer.value, decimals);
    let from = format_address(&transfer.from);
    let to = format_address(&transfer.to);

    format!(
        "{} {} {} -> {} (block: {}, tx: 0x{}..., log_index: {})",
        amount,
        symbol,
        from,
        to,
        transfer.block_number,
        transfer.transaction_hash,
        transfer.log_index
    )
}

async fn erc20_monitor_exex<Node>(mut ctx: ExExContext<Node>) -> eyre::Result<()>
where Node: FullNodeComponents<Types: NodeTypes<Primitives = EthPrimitives>> {
    // Example: set up a filter for large transfers
    let filter = TransferFilter::LargeTransfers(U256::from(1_000_000u64));
    let mut detector = TransferDetector::new(filter.clone());

    while let Some(notification) = ctx.notifications.try_next().await? {
        match &notification {
            ExExNotification::ChainCommitted {new} => {
                info!("committed chain = {:?}", new.range());
            }
            ExExNotification::ChainReorged { old, new } => {
                info!("transfers may have changed; from_chain = {:?}, to_chain = {:?}", old.range(), new.range());
            }
            ExExNotification::ChainReverted { old } => {
                info!("some transfers were removed. Reverted chain = {:?}", old.range());
            }
        }

        if let Some(committed_chain) = notification.committed_chain() {
            // Process the committed chain for transfer events
            process_committed_chain(&*committed_chain, &detector, &filter).await;
            ctx.events.send(ExExEvent::FinishedHeight(committed_chain.tip().num_hash()))?;
        }
    }

    Ok(())
}

async fn process_committed_chain(chain: &Chain, detector: &TransferDetector, filter: &TransferFilter) {
    let blocks = chain.blocks();
    info!("Processing committed chain with {} blocks", blocks.len());

    for (_, block) in blocks {
        let block_number = block.number;
        let timestamp = block.timestamp;
        let block_hash = block.hash();

        info!("Processing block {} (hash: {:?})", block_number, block_hash);

        let receipts = chain.receipts_by_block_hash(block_hash);

        if let Some(receipts) = receipts {
            let tx_hash = blocks[&block_number].hash();
            // Flatten all transfer events from all receipts into a single Vec<TransferEvent>
            let transfers: Vec<TransferEvent> = receipts.iter()
                .flat_map(|receipt| {
                    receipt.logs.iter()
                        .filter(|log| TransferDetector::is_transfer_log(log))
                        .filter_map(|log| TransferDetector::parse_transfer_log(log, block_number, tx_hash))
                })
                .collect();

            for transfer in &transfers {
                if apply_transfer_filter(transfer, filter) {
                    info!(
                        "Interesting transfer detected: {}",
                        format_transfer(transfer)
                    );
                }
            }
        }
    }
}

fn main() -> eyre::Result<()> {
    reth::cli::Cli::parse_args().run(async move |builder, _| {
        let handle = builder.node(EthereumNode::default())
        .install_exex("erc20_monitor_exex", async move |ctx| Ok(erc20_monitor_exex(ctx)))
        .launch().await?;

        handle.wait_for_node_exit().await
    })
}
