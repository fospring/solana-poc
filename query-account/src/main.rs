use solana_client::rpc_client::RpcClient;
use spl_token::{solana_program::{program_pack::Pack, pubkey::Pubkey}, state::Account as SplAccount};
use std::str::FromStr;

fn main() -> anyhow::Result<()> {
    let url = "https://api.mainnet-beta.solana.com".to_string();
    let client = RpcClient::new(url);
    let alice_pubkey = Pubkey::from_str("3emsAVdmGKERbHjmGfQ6oZ1e35dkf5iYcS6U4CPKFVaa").unwrap();
    let account = client.get_account(&alice_pubkey)?;
    println!("account info: {:?}", account);
    let spl_data = SplAccount::unpack(&account.data).unwrap();
    println!("spl_data info: {:?}", spl_data);
    Ok(())
}
