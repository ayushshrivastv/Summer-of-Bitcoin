#![allow(unused)]
use bitcoincore_rpc::{Auth, Client, RpcApi};
use serde::Deserialize;
use serde_json::json;


const RPC_URL: &str = "http://127.0.0.1:18443";
const RPC_USER: &str = "alice";
const RPC_PASS: &str = "password";

fn send(rpc: &Client, addr: &str) -> bitcoincore_rpc::Result<String> {
    let args = [
        json!([{ addr: 100 }]),
        json!(null),
        json!(null),
        json!(null),
        json!(null),
    ];
    #[derive(Deserialize)]
    struct SendResult {
        complete: bool,
        txid: String,
    }
    let send_result = rpc.call::<SendResult>("send", &args)?;
    assert!(send_result.complete);
    Ok(send_result.txid)
}

fn list_wallet_dir(client: &Client) -> bitcoincore_rpc::Result<Vec<String>> {
    #[derive(Deserialize)]
    struct Name {
        name: String,
    }
    #[derive(Deserialize)]
    struct CallResult {
        wallets: Vec<Name>,
    }
    let result: CallResult = client.call("listwalletdir", &[])?;
    Ok(result.wallets.into_iter().map(|n| n.name).collect())
}

fn main() -> bitcoincore_rpc::Result<()> {
    let rpc = Client::new(
        RPC_URL,
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )?;
    let info = rpc.get_blockchain_info()?;
    println!("{:?}", info);

    //Create or load the wallet
    let wallets = list_wallet_dir(&rpc)?;
    if !wallets.contains(&"testwallet".to_string()) {
        let create_wallet: serde_json::Value =
            rpc.call("createwallet", &[json!("testwallet")])?;
        println!("{:?}", create_wallet);
    } else {
        println!("Wallet exists");
    }
    let wallet_url = format!("{}/wallet/{}", RPC_URL, "testwallet");
    let wallet_rpc = Client::new(
        &wallet_url,
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )?;

    //Generate a new address
    let new_addr = wallet_rpc.get_new_address(None, None)?;
    println!("{:?}", new_addr);
    let checked_addr: bitcoincore_rpc::bitcoin::Address =
        new_addr.to_string().parse().expect("Failed to parse address");

    //Mine 103 blocks to the new address
    let blocks = wallet_rpc.generate_to_address(103, &checked_addr)?;
    println!("Mined {} blocks", blocks.len());

    //Prepare a transaction to send 100 BTC with an OP_RETURN output
    let op_return_hex = "57652061726520616c6c205361746f7368692121";
    let outputs = json!({
        "bcrt1qq2yshcmzdlznnpxx258xswqlmqcxjs4dssfxt2": 100.0,
        "data": op_return_hex
    });
    let raw_tx: String =
        wallet_rpc.call("createrawtransaction", &[json!([]), outputs])?;
    println!("Raw tx: {}", raw_tx);

    #[derive(Deserialize)]
    struct FundResult {
        hex: String,
        fee: f64,
        changepos: i32,
    }
    let fund_options = json!({ "feeRate": 0.00021 });
    let fund_result: FundResult =
        wallet_rpc.call("fundrawtransaction", &[json!(raw_tx), fund_options])?;
    println!("Funded tx: {}", fund_result.hex);

    #[derive(Deserialize)]
    struct SignResult {
        hex: String,
        complete: bool,
    }
    let sign_result: SignResult =
        wallet_rpc.call("signrawtransactionwithwallet", &[json!(fund_result.hex)])?;
    assert!(sign_result.complete, "Signing incomplete");
    println!("Signed tx: {}", sign_result.hex);

    //Send the transaction
    let txid: String =
        wallet_rpc.call("sendrawtransaction", &[json!(sign_result.hex)])?;
    println!("Txid: {}", txid);

    //Write the txid to out.txt
    let mut file = File::create("out.txt")?;
    file.write_all(txid.as_bytes())?;
    Ok(())
}
