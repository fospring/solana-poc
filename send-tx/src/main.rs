use std::{str::FromStr};

use anyhow::Result;
use borsh::{BorshSerialize, BorshDeserialize};
use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::{
     instruction::Instruction,
     pubkey::Pubkey,
     signature::{Keypair, Signer},
     transaction::Transaction,
     signature::read_keypair_file,
};
// A custom program instruction. This would typically be defined in
// another crate so it can be shared between the on-chain program and
// the client.
#[derive(BorshSerialize, BorshDeserialize)]
enum BankInstruction {
    Initialize,
    Deposit { lamports: u64 },
    Withdraw { lamports: u64 },
}

fn send_hello_tx(
    client: &RpcClient,
    program_id: Pubkey,
    payer: &Keypair
) -> Result<()> {
    let bank_instruction = BankInstruction::Initialize;
    let instruction = Instruction::new_with_borsh(
        program_id,
        &bank_instruction,
        vec![],
    );
    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[payer],
        blockhash,
    );
    let result = client.send_and_confirm_transaction(&tx)?;
    println!("result: {:?}", result);
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let rpc_client = RpcClient::new("http://127.0.0.1:8899");
    let program_id = Pubkey::from_str("24bUpv6ppELeWpwkhwwefm5V9Dd2RobqQvuQ1YWDA7qn")?;
    let key_pair = read_keypair_file("/Users/qiuyongchun/.config/solana/id.json").map_err(|err| {
        anyhow::anyhow!("read key pair error: {:?}", err)
    })?;
    send_hello_tx(
        &rpc_client,
        program_id,
        &key_pair,
    )?;
    Ok(())
}