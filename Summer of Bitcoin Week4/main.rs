use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;

use miniscript::descriptor::DescriptorPublicKey;
use miniscript::Descriptor;
use miniscript::bitcoin::Network;
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct AddressStats {
    funded_txo_sum: u64,
    spent_txo_sum: u64,
    #[serde(rename = "tx_count")]
    _tx_count: u64,
}

#[derive(Deserialize)]
struct AddressInfo {
    address: String,
    chain_stats: AddressStats,
    mempool_stats: AddressStats,
}

fn main() -> Result<(), Box<dyn Error>> {
    let descriptor_str = "wpkh(tpubD6NzVbkrYhZ4XgiXtGrdW5XDAPFCL9h7we1vwNCpn8tGbBcgfVYjXyhWo4E1xkh56hjod1RhGjxbaTLV3X4FyWuejifB9jusQ46QzG87VKp/*)#adv567t2";
    let descriptor: Descriptor<DescriptorPublicKey> = Descriptor::from_str(descriptor_str)?;
    let client = Client::new();
    let base_url = "http://localhost:8094/testnet/api";
    let mut total_satoshis: u64 = 0;

    for index in 0..100 {
        let derived_descriptor = descriptor.at_derivation_index(index);
        let address = derived_descriptor.address(Network::Testnet)?;
        let addr_str = address.to_string();
        let url = format!("{}/address/{}", base_url, addr_str);

        let resp = client.get(&url).send()?;
        if !resp.status().is_success() {
            eprintln!("Warning: Failed to fetch data for address {}. Skipping.", addr_str);
            continue;
        }

        let addr_info: AddressInfo = resp.json()?;
        let balance_satoshis = (addr_info.chain_stats.funded_txo_sum
            .saturating_sub(addr_info.chain_stats.spent_txo_sum))
            + (addr_info.mempool_stats.funded_txo_sum
                .saturating_sub(addr_info.mempool_stats.spent_txo_sum));

        total_satoshis += balance_satoshis;
    }

    let total_btc = total_satoshis as f64 / 100_000_000.0;
    let mut file = File::create("out.txt")?;
    writeln!(file, "{:.8}", total_btc)?;

    Ok(())
}
