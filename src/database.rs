// use sqlx::SqlitePool;
// use std::error::Error;
// use crate::event::Erc20Transfer;

// pub struct Database {
//     pool: SqlitePool
// }

// impl Database {
//     pub async fn new(&self) -> Result<Self, Box<dyn Error>> {
//         let pool = SqlitePool::connect("sqlite:erc20_exex_tracker.db").await?;

//         sqlx::query(
//             r#"
//             CREATE TABLE IF NOT EXISTS transfers (
//                 id INTEGER PRIMARY KEY AUTOINCREMENT,
//                 transaction_hash TEXT NOT NULL,
//                 block_number INTEGER NOT NULL,
//                 from_address TEXT NOT NULL,
//                 to_address TEXT NOT NULL,
//                 amount TEXT NOT NULL,
//                 token_contract TEXT NOT NULL,
//                 timestamp INTEGER NOT NULL
//             )
//             "#
//         )
//         .execute(&pool).await?;

//         Ok(Database { pool })
//     }

//     async fn insert_transfer(&self, transfer: &Erc20Transfer) -> Result<(), Box<dyn Error>> {
//         sqlx::query(
//             r#"
//             INSERT INTO transfers (transaction_hash, block_number, from_address, to_address, amount, token_contract, timestamp)
//             VALUES (?, ?, ?, ?, ?, ?, ?)
//             "#
//         )
//         .bind(&transfer.transaction_hash)
//         .bind(transfer.block_number as i64)
//         .bind(&transfer.from_address)
//         .bind(&transfer.to_address)
//         .bind(&transfer.amount)
//         .bind(&transfer.token_contract)
//         .bind(transfer.timestamp as i64)
//         .execute(&self.pool).await?;
//         Ok(())
//     }
// }