use std::{thread, time::Duration};

use solana_client::rpc_client::RpcClient;
use solana_client::{
    pubsub_client::PubsubClient,
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::commitment_config::CommitmentConfig;

fn main() {
    println!("Hello, world!");

    let config = RpcTransactionLogsConfig {
        commitment: Some(CommitmentConfig::processed()),
    };
    let (mut cli, reciver) = PubsubClient::logs_subscribe(
        "ws://127.0.0.1:8900",
        RpcTransactionLogsFilter::Mentions(vec![
            "Hop5JPRNK77KsGFAqn5iCExYBYPF3jn2dPV2k1sEFK3y".to_string()
        ]),
        config,
    )
    .unwrap();

    let mut counter = 0;
    let mut slots: Vec<u64> = vec![];
    loop {
        let data = reciver.recv().unwrap();
        slots.push(data.context.slot);
        println!(
            "slot: {}, is_err: {}, signature: {}, logs: {:?}",
            data.context.slot,
            data.value.err.is_some(),
            data.value.signature.clone(),
            data.value.logs
        );

        counter += 1;
        if counter > 1 {
            break;
        }
    }

    thread::sleep(Duration::from_secs(20));
    let url = "http://127.0.0.1:8899".to_string();
    let client = RpcClient::new(url);
    for slot in slots {
        let block = client.get_block(slot).unwrap();
        println!("block: {:?}", block);
        for tx in block.transactions {
            println!(
                "tx in block: {} transactions: {:?}",
                block.parent_slot + 1,
                tx
            );
            if let Some(meta) = tx.meta {
                println!("meta.log_messages: {:?}", meta.log_messages);
            }
        }
    }

    cli.shutdown().unwrap();
}
