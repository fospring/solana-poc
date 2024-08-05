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
    loop {
        let data = reciver.recv().unwrap();
        println!("data: {:?}", data);
        counter += 1;
        if counter > 2 {
            break;
        }
    }

    cli.shutdown().unwrap();
}
