use std::collections::VecDeque;
use std::str::FromStr;
use std::future::Future;

use tendermint::abci::Event;
use tendermint::block::Height;
use tendermint_rpc::Order;
use tendermint_rpc::{query::Query, Client, HttpClient};

const LOCAL_RPC: &str = "http://127.0.0.1:26657";
const JUNO_RPC: &str = "https://rpc-juno.itastakers.com";

#[tokio::main]
async fn main() {
    let mut queue: VecDeque<Height> = VecDeque::new();
    let client = HttpClient::new(JUNO_RPC).unwrap();

    while let Ok(latest_block) = client.latest_block().await {
        let latest_block_height = latest_block.block.header.height;
        match queue.len() {
            0 => queue.push_back(latest_block_height),
            1..=5 => {
                let n = queue.len();
                if latest_block_height != queue[n - 1] {
                    queue.push_back(latest_block_height);
                }
            }
            _ => {
                let bh = queue.pop_front().unwrap();
                show_wasm_events(&client, bh).await;
            }
        }
    }
    println!("Final: {:?}", queue);
}

async fn show_wasm_events(client: &HttpClient, height: Height) {
    println!("Transaction height::: {}", height.value());
    let searched_tx = client
        .tx_search(
            Query::from_str(format!("tx.height = {}", height.value()).as_str()).unwrap(),
            false,
            1,
            100,
            Order::Ascending,
        )
        .await
        .unwrap();

    if searched_tx.txs.len() > 0 {
        println!("Searched txs counts: {:?}", searched_tx.total_count);
        let tx_count = searched_tx.total_count;
        for i in 0..tx_count {
            let tx = &searched_tx.txs[i as usize];
            let wasm_events = tx
                .tx_result
                .events
                .iter()
                .filter(|v| v.type_str == *"wasm")
                .map(|v| v.clone())
                .collect::<Vec<Event>>();
            if !wasm_events.is_empty() {
                println!("Filtered wasm events: {:?}", wasm_events);
            }
        }
    }
}
