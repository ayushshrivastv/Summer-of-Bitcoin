use byteorder::{LittleEndian, WriteBytesExt};
use hex::{decode, encode};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

const DIFFICULTY_TARGET: &str = "0000ffff00000000000000000000000000000000000000000000000000000000";
const PREV_BLOCK_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const COINBASE_REWARD: u64 = 5_000_000_000;
const MEMPOOL_DIR: &str = "mempool";
const VERSION: u32 = 4;
const BITS: u32 = 0x1f00ffff;
const MAX_BLOCK_WEIGHT: u64 = 4_000_000;

fn double_sha256(data: &[u8]) -> Vec<u8> {
    let first = Sha256::digest(data);
    Sha256::digest(&first).to_vec()
}

fn write_varint(n: u64, buffer: &mut Vec<u8>) {
    if n < 0xfd {
        buffer.push(n as u8);
    } else if n <= 0xffff {
        buffer.push(0xfd);
        buffer.write_u16::<LittleEndian>(n as u16).unwrap();
    } else if n <= 0xffffffff {
        buffer.push(0xfe);
        buffer.write_u32::<LittleEndian>(n as u32).unwrap();
    } else {
        buffer.push(0xff);
        buffer.write_u64::<LittleEndian>(n).unwrap();
    }
}

//coinbase transaction using the provided witness root.
fn serialize_coinbase_transaction(witness_root: &str) -> io::Result<Vec<u8>> {
    let mut buffer = Vec::new();

    buffer.write_u32::<LittleEndian>(VERSION)?;
    buffer.push(0x00); //SegWitmarker
    buffer.push(0x01); //SgWitflag

    write_varint(1, &mut buffer);
    buffer.extend(vec![0u8; 32]);
    buffer.write_u32::<LittleEndian>(0xffffffff)?;

    let height = 0x1d15e6; //block height for signatuture issue..
    let script = vec![
        0x03,
        (height & 0xff) as u8,
        ((height >> 8) & 0xff) as u8,
        ((height >> 16) & 0xff) as u8,
    ];
    write_varint(script.len() as u64, &mut buffer);
    buffer.extend(&script);
    buffer.write_u32::<LittleEndian>(0xffffffff)?;

    write_varint(2, &mut buffer); // Output count

    buffer.write_u64::<LittleEndian>(COINBASE_REWARD)?;
    let pk_script = decode("76a914000000000000000000000000000000000000000088ac")
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    write_varint(pk_script.len() as u64, &mut buffer);
    buffer.extend(&pk_script);

    buffer.write_u64::<LittleEndian>(0)?;

    //hashing the witness root..
    let witness_root_bytes =
        decode(witness_root).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let witness_reserved = vec![0u8; 32];
    let mut combined = witness_root_bytes.clone();
    combined.extend_from_slice(&witness_reserved);
    let commitment_hash = double_sha256(&combined);
    let commitment_hex = encode(commitment_hash);
    let commitment = format!("6a24aa21a9ed{}", commitment_hex);
    let commitment_bytes =
        decode(&commitment).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    write_varint(commitment_bytes.len() as u64, &mut buffer);
    buffer.extend(&commitment_bytes);

    write_varint(1, &mut buffer); // Witness count
    write_varint(witness_reserved.len() as u64, &mut buffer);
    buffer.extend(&witness_reserved);

    buffer.write_u32::<LittleEndian>(0)?;

    Ok(buffer)
}

//returns the coinbase transaction ID...
fn coinbase_txid(serialized: &[u8]) -> String {
    encode(double_sha256(serialized))
}

//builds the merkle root from a txids..
fn build_merkle_root(txids: &[String]) -> io::Result<String> {
    if txids.is_empty() {
        return Ok("0".repeat(64));
    }
    let mut current: Vec<Vec<u8>> = txids
        .iter()
        .map(|txid| {
            let mut bytes =
                decode(txid).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            bytes.reverse();
            Ok(bytes)
        })
        .collect::<io::Result<Vec<Vec<u8>>>>()?;

    while current.len() > 1 {
        let mut next = Vec::new();
        for i in (0..current.len()).step_by(2) {
            let left = &current[i];
            let right = if i + 1 < current.len() {
                &current[i + 1]
            } else {
                left
            };
            let mut combined = Vec::new();
            combined.extend_from_slice(left);
            combined.extend_from_slice(right);
            next.push(double_sha256(&combined));
        }
        current = next;
    }

    let mut root = current[0].clone();
    root.reverse(); // Convert back to big-endian for output
    Ok(encode(root))
}

//merkle root wtxids..
fn build_witness_merkle_root(wtxids: &[String]) -> io::Result<String> {
    if wtxids.is_empty() {
        return Ok("0".repeat(64));
    }
    let mut current: Vec<Vec<u8>> = wtxids
        .iter()
        .map(|wtxid| decode(wtxid).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)))
        .collect::<io::Result<Vec<Vec<u8>>>>()?;

    while current.len() > 1 {
        let mut next = Vec::new();
        for i in (0..current.len()).step_by(2) {
            let left = &current[i];
            let right = if i + 1 < current.len() {
                &current[i + 1]
            } else {
                left
            };
            let mut combined = Vec::new();
            combined.extend_from_slice(left);
            combined.extend_from_slice(right);
            next.push(double_sha256(&combined));
        }
        current = next;
    }

    Ok(encode(current[0].clone()))
}

//continous minning until the hash meets the difficulty target.
fn mine_block(header_start: &[u8], difficulty_target: &str) -> io::Result<(u32, String)> {
    let target =
        decode(difficulty_target).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let mut nonce = 0u32;
    loop {
        let mut header = header_start.to_vec();
        header.write_u32::<LittleEndian>(nonce)?;
        let block_hash = double_sha256(&header);
        let hash_be: Vec<u8> = {
            let mut hb = block_hash.clone();
            hb.reverse();
            hb
        };
        if hash_be < target {
            return Ok((nonce, encode(block_hash)));
        }
        nonce = nonce
            .checked_add(1)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Nonce overflow"))?;
    }
}

//block header.
fn serialize_header(
    version: u32,
    prev_hash: &str,
    merkle_root: &str,
    time: u32,
    bits: u32,
    nonce: u32,
) -> io::Result<Vec<u8>> {
    let mut header = Vec::new();
    header.write_u32::<LittleEndian>(version)?;
    let mut prev_hash_bytes =
        decode(prev_hash).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    prev_hash_bytes.reverse();
    header.extend(prev_hash_bytes);
    let mut merkle_bytes =
        decode(merkle_root).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    merkle_bytes.reverse();
    header.extend(merkle_bytes);
    header.write_u32::<LittleEndian>(time)?;
    header.write_u32::<LittleEndian>(bits)?;
    header.write_u32::<LittleEndian>(nonce)?;
    Ok(header)
}

//transaction
fn validate_transaction(tx: &Value) -> bool {
    tx.get("vin").is_some()
        && tx.get("vout").is_some()
        && tx["vin"].as_array().map_or(false, |v| !v.is_empty())
        && tx["vout"].as_array().map_or(false, |v| !v.is_empty())
        && tx.get("hex").is_some()
        && tx.get("weight").is_some()
}

fn main() -> io::Result<()> {
    //transactions with weights.
    let mut txs = Vec::new();
    for entry in fs::read_dir(MEMPOOL_DIR)? {
        let path = entry?.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            let file = File::open(&path)?;
            let tx: Value = serde_json::from_reader(file)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            if validate_transaction(&tx) {
                let filename = path
                    .file_stem()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid file stem"))?
                    .to_str()
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "File stem not valid UTF-8")
                    })?
                    .to_string();
                let hex = tx["hex"]
                    .as_str()
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "Transaction hex missing")
                    })?
                    .to_string();
                let weight = tx["weight"].as_u64().ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Transaction weight missing or invalid",
                    )
                })?;
                let tx_bytes =
                    decode(&hex).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                let computed_txid = encode(double_sha256(&tx_bytes));
                if filename != computed_txid {
                    println!(
                        "Warning: Filename {} != computed txid {}",
                        filename, computed_txid
                    );
                }
                txs.push((filename, hex, weight));
            }
        }
    }

    //transactions by weight...
    let mut txids = Vec::new();
    let mut tx_hexes = Vec::new();
    let mut total_weight = 0;
    for (txid, hex, weight) in txs {
        if total_weight + weight + 1000 <= MAX_BLOCK_WEIGHT {
            //reserve weight
            total_weight += weight;
            txids.push(txid);
            tx_hexes.push(hex);
        }
    }
    println!(
        "Selected txids (natural order): {:?}, Total weight: {}",
        txids, total_weight
    );

    //dummy coinbase..
    let mut witness_root = String::new();

    // Two iterations to stabilize the witness root (handling coinbase dependency)
    for _ in 0..2 {
        let _ = serialize_coinbase_transaction(&witness_root)?;
        let mut wtxids =
            vec!["0000000000000000000000000000000000000000000000000000000000000000".to_string()];
        for hex in &tx_hexes {
            let tx_bytes =
                decode(hex).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            wtxids.push(encode(double_sha256(&tx_bytes)));
        }
        witness_root = build_witness_merkle_root(&wtxids)?;
    }

    //final coinbase transaction
    let final_coinbase_serialized = serialize_coinbase_transaction(&witness_root)?;
    let final_coinbase_txid = coinbase_txid(&final_coinbase_serialized);
    let mut all_txids = vec![final_coinbase_txid.clone()];
    all_txids.extend(txids.clone());

    //build the merkle root using all txids..
    let merkle_root = build_merkle_root(&all_txids)?;

    //block minning.
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        .as_secs() as u32;
    let header_start =
        serialize_header(VERSION, PREV_BLOCK_HASH, &merkle_root, timestamp, BITS, 0)?;
    let (nonce, _block_hash) = mine_block(&header_start[..76], DIFFICULTY_TARGET)?;
    let block_header = serialize_header(
        VERSION,
        PREV_BLOCK_HASH,
        &merkle_root,
        timestamp,
        BITS,
        nonce,
    )?;

    //output.
    let mut file = File::create("out.txt")?;
    writeln!(file, "{}", encode(&block_header))?;
    writeln!(file, "{}", encode(&final_coinbase_serialized))?;
    for txid in &all_txids {
        writeln!(file, "{}", txid)?;
    }

    println!("Block mined successfully! Output written to out.txt");
    Ok(())
}
