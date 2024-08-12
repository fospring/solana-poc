use std::{thread, time::Duration};

use anchor_lang::prelude::*;
use anchor_lang::{AnchorDeserialize, AnchorSerialize};
use solana_client::rpc_client::RpcClient;
use solana_client::{
    pubsub_client::PubsubClient,
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_transaction_status::EncodedTransaction;

#[derive(Debug)]
#[event]
pub struct AccountWithdrawSol {
    pub account_id: [u8; 32],
    pub sender: [u8; 32],
    pub receiver: [u8; 32],
    pub broker_hash: [u8; 32],
    pub token_hash: [u8; 32],
    pub token_amount: u128,
    pub fee: u128,
    pub chain_id: u128,
    pub withdraw_nonce: u64,
}

pub fn to_bytes32(bytes: &[u8]) -> [u8; 32] {
    let mut bytes32 = [0u8; 32];
    // add ledding zeros to the bytes
    bytes32[32 - bytes.len()..].copy_from_slice(bytes);
    bytes32
}

impl AccountWithdrawSol {
    pub fn encode(&self) -> Vec<u8> {
        let mut encoded = Vec::new();
        encoded.extend_from_slice(&self.account_id);
        encoded.extend_from_slice(&self.sender);
        encoded.extend_from_slice(&self.receiver);
        encoded.extend_from_slice(&self.broker_hash);
        encoded.extend_from_slice(&self.token_hash);
        encoded.extend_from_slice(&to_bytes32(&self.token_amount.to_be_bytes()));
        encoded.extend_from_slice(&to_bytes32(&self.fee.to_be_bytes()));
        encoded.extend_from_slice(&to_bytes32(&self.chain_id.to_be_bytes()));
        encoded.extend_from_slice(&to_bytes32(&self.withdraw_nonce.to_be_bytes()));
        encoded
    }

    pub fn decode(encoded: &[u8]) -> anyhow::Result<Self> {
        let mut offset = 0;
        let account_id = encoded[offset..offset + 32].try_into().unwrap();
        offset += 32;
        let sender = encoded[offset..offset + 32].try_into().unwrap();
        offset += 32;
        let receiver = encoded[offset..offset + 32].try_into().unwrap();
        offset += 32;
        let broker_hash = encoded[offset..offset + 32].try_into().unwrap();
        offset += 32;
        let token_hash = encoded[offset..offset + 32].try_into().unwrap();
        offset += 32;
        // higher 128 bits of the token amount
        let token_amount =
            u128::from_be_bytes(encoded[offset + 16..offset + 32].try_into().unwrap());
        offset += 32;
        let fee = u128::from_be_bytes(encoded[offset + 16..offset + 32].try_into().unwrap());
        offset += 32;
        let chain_id = u128::from_be_bytes(encoded[offset + 16..offset + 32].try_into().unwrap());
        offset += 32;
        let withdraw_nonce =
            u64::from_be_bytes(encoded[offset + 24..offset + 32].try_into().unwrap());
        Ok(Self {
            account_id,
            sender,
            receiver,
            broker_hash,
            token_hash,
            token_amount,
            fee,
            chain_id,
            withdraw_nonce,
        })
    }
}

fn main() {
    println!("Hello, world!");

    let config = RpcTransactionLogsConfig {
        commitment: Some(CommitmentConfig::processed()),
    };
    let (mut cli, reciver) = PubsubClient::logs_subscribe(
        "ws://127.0.0.1:8900",
        RpcTransactionLogsFilter::Mentions(vec![
            "24bUpv6ppELeWpwkhwwefm5V9Dd2RobqQvuQ1YWDA7qn".to_string()
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
    cli.shutdown().unwrap();

    thread::sleep(Duration::from_secs(20));
    let url = "http://127.0.0.1:8899".to_string();
    let client = RpcClient::new(url);
    for slot in slots {
        let block = client.get_block(slot).unwrap();
        println!("block: {:?}", block);
        for tx in block.transactions {
            match tx.transaction {
                EncodedTransaction::Json(trans) => {
                    if let Some(meta) = &tx.meta {
                        if let Some(err) = &meta.err {
                            println!("signature: {}, err: {}", trans.signatures[0].clone(), err);
                        } else {
                            println!(
                                "tx in block: {} signature: {:?}",
                                block.parent_slot + 1,
                                trans.signatures[0].clone()
                            );
                            if let Some(meta) = tx.meta {
                                println!(
                                    "signature: {}, meta.log_messages: {:?}",
                                    trans.signatures[0].clone(),
                                    meta.log_messages
                                );
                            }
                        }
                    }
                }
                EncodedTransaction::LegacyBinary(_) => todo!(),
                EncodedTransaction::Binary(_, _) => todo!(),
                EncodedTransaction::Accounts(_) => todo!(),
            }
        }
    }
    // todo: query tx status after fetch the logs
}

#[cfg(test)]
mod tests {
    use super::AccountWithdrawSol;
    use anchor_lang::AnchorDeserialize;
    use base64::prelude::*;
    use solana_client::rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient};
    use solana_sdk::{pubkey, signature::Signature};
    use solana_transaction_status::UiTransactionEncoding;
    use std::str::FromStr;

    #[test]
    fn test_account_withdraw_sol_decode() {
        let account_withdraw_sol = AccountWithdrawSol {
            account_id: [1u8; 32],
            sender: [2u8; 32],
            receiver: [3u8; 32],
            broker_hash: [4u8; 32],
            token_hash: [5u8; 32],
            token_amount: 1000,
            fee: 10,
            chain_id: 1,
            withdraw_nonce: 1,
        };
        // let mut buffer: Vec<u8> = Vec::new();
        let encoded = anchor_lang::Event::data(&account_withdraw_sol);
        // account_withdraw_sol.serialize(&mut buffer).unwrap();
        let data = BASE64_STANDARD.encode(encoded);
        println!("{}", data);

        let raw = "tc6PPKaR1tUBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUF6AMAAAAAAAAAAAAAAAAAAAoAAAAAAAAAAAAAAAAAAAABAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAA=";
        assert_eq!(data, raw);

        let data = BASE64_STANDARD.decode(data).unwrap();
        let discriminator_preimage = format!("event:AccountWithdrawSol").into_bytes();
        let discriminator = anchor_syn::hash::hash(&discriminator_preimage);
        let discriminator = format!("{}", hex::encode(&discriminator.0[..8]));

        let data = data[8..].to_vec();
        let withdraw = AccountWithdrawSol::deserialize(&mut &data[..]).unwrap();
        println!(
            "discriminator: {}, withdraw data: {:?}",
            discriminator, withdraw
        );
    }

    #[ignore]
    #[test]
    fn test_get_transaction() {
        let url = "https://api.devnet.solana.com".to_string();
        // https://explorer.solana.com/tx/3njCSYscVEoJiBmm9TtaYoxi29oHFKZa5ggyCek6JvFaRAiEBLMb3F5x8XZnsBB3zGuPirFaZacRue5Mo7rjq37a?cluster=devnet
        let signature = Signature::from_str("3njCSYscVEoJiBmm9TtaYoxi29oHFKZa5ggyCek6JvFaRAiEBLMb3F5x8XZnsBB3zGuPirFaZacRue5Mo7rjq37a").unwrap();
        let client = RpcClient::new(url);
        let tx = client
            .get_transaction(&signature, UiTransactionEncoding::Json)
            .unwrap();
        println!("tx={:?}", tx);
    }

    // https://solana.com/docs/rpc/http/getsignaturesforaddress
    #[test]
    fn test_get_signatures_for_address() {
        let url = "https://api.devnet.solana.com".to_string();
        let client = RpcClient::new(url);
        // slot 317,319,257, which is bigger than 317319256
        let signature = Signature::from_str("3CJi8sDTr1jBpfYwuufDd7wAqiSgRPrqSaNDDuxiTh53UbyVQ5QRoQUyDb49CQoWbK2DtfafPr9ufm9puHZvR9kW").unwrap();
        let config = GetConfirmedSignaturesForAddress2Config {
            before: Some(signature),
            until: None,
            limit: Some(1000),
            commitment: None,
        };
        let address = pubkey!("2vauk9Xi84cehajW8HKdWw7nAtcjBTuQCjPPToxb5UwM");
        let txs = client
            .get_signatures_for_address_with_config(&address, config)
            .unwrap();
        println!("txs={:?}", txs);
        // txs=[
        // RpcConfirmedTransactionStatusWithSignature { signature: "4RnrTbMg2Xo1CJyGAd7c5WpPqU5ZiysUHZtHUxJmGYukGYXDGQhpKuANsSwhyK5K5jvadBZaw7L7XD7XadGnxCAb", slot: 317319256, err: None, memo: None, block_time: Some(1723102894), confirmation_status: Some(Finalized) },
        // RpcConfirmedTransactionStatusWithSignature { signature: "NasfQm8d6cG3UisM8RurNuxNrnXWZhJycSaoXEv8ajnXE5BiFhYZBJr6zvsodjv8wcXDbzWLDzkNRmgGZYvBpWU", slot: 317096934, err: None, memo: None, block_time: Some(1723020084), confirmation_status: Some(Finalized) },
        // RpcConfirmedTransactionStatusWithSignature { signature: "3njCSYscVEoJiBmm9TtaYoxi29oHFKZa5ggyCek6JvFaRAiEBLMb3F5x8XZnsBB3zGuPirFaZacRue5Mo7rjq37a", slot: 317096464, err: None, memo: None, block_time: Some(1723019910), confirmation_status: Some(Finalized) },
        // RpcConfirmedTransactionStatusWithSignature { signature: "3zV1eidg5Taobt8fG5LBdqaGS5LHb55yWYQBNMU1gf1EZa8Eivsvqiogct2Q3hbvHGBg5vV9529zp73e2ZcXxbe3", slot: 317090617, err: None, memo: None, block_time: Some(1723017729), confirmation_status: Some(Finalized) }
        //]
        assert_eq!(txs.len(), 4);

        // slot 317090616 is smaller than 317090617
        let signature = Signature::from_str("5vAUjVkiAeFMFGbWn5Rvt9qozLyXpkGEwENdaffjvpwaqep7ss15LemxyUo5QzCmVDFuK5Fqcru3defXVCii1TeA").unwrap();
        let config = GetConfirmedSignaturesForAddress2Config {
            before: None,
            until: Some(signature),
            limit: Some(1000),
            commitment: None,
        };
        let address = pubkey!("2vauk9Xi84cehajW8HKdWw7nAtcjBTuQCjPPToxb5UwM");
        let txs2 = client
            .get_signatures_for_address_with_config(&address, config)
            .unwrap();
        println!("txs2={:?}", txs2);
        assert_eq!(txs2.len(), 4);
    }

    #[ignore]
    #[test]
    fn test_local_get_signatures_for_address() {
        let url = "http://127.0.0.1:8899".to_string();
        let client = RpcClient::new(url);
        let config = GetConfirmedSignaturesForAddress2Config {
            before: None,
            until: None,
            limit: Some(1000),
            commitment: None,
        };
        let address = pubkey!("24bUpv6ppELeWpwkhwwefm5V9Dd2RobqQvuQ1YWDA7qn");
        let txs = client
            .get_signatures_for_address_with_config(&address, config)
            .unwrap();
        println!("txs in address: {:?} are: {:?}", address, txs);
    }
}
