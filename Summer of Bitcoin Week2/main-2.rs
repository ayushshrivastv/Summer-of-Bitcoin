use bitcoin::{
    blockdata::{
        script::Script,
        transaction::{OutPoint, Transaction, TxIn, TxOut},
        witness::Witness,
    },
    consensus::encode::serialize_hex,
    hash_types::WScriptHash,
    hashes::{sha256, Hash},
    network::constants::Network,
    secp256k1::{Message, Secp256k1, SecretKey},
    util::{amount::Amount, sighash::EcdsaSighashType, sighash::SighashCache},
    Address, PrivateKey, Txid,
};
use std::fs;
use std::str::FromStr;

fn main() {
    println!("Starting transaction creation...");

    //Initialize secp256k1 context for signing
    let secp = Secp256k1::new();

    //private keys from hex
    let privkey1_hex = "39dc0a9f0b185a2ee56349691f34716e6e0cda06a7f9707742ac113c4e2317bf";
    let privkey2_hex = "5077ccd9c558b7d04a81920d38aa11b4a9f9de3b23fab45c3ef28039920fdd6d";
    let secret_key1 = SecretKey::from_slice(&hex::decode(privkey1_hex).unwrap()).unwrap();
    let secret_key2 = SecretKey::from_slice(&hex::decode(privkey2_hex).unwrap()).unwrap();
    let privkey1 = PrivateKey::new(secret_key1, Network::Bitcoin);
    let privkey2 = PrivateKey::new(secret_key2, Network::Bitcoin);
    println!("Private keys parsed.");

    //script
    let witness_script_hex = "5221032ff8c5df0bc00fe1ac2319c3b8070d6d1e04cfbf4fedda499ae7b775185ad53b21039bbc8d24f89e5bc44c5b0d1980d6658316a6b2440023117c3c03a4975b04dd5652ae";
    let witness_script_bytes =
        hex::decode(witness_script_hex).expect("Invalid hex in witness script");
    let witness_script = Script::from(witness_script_bytes);
    println!("Witness script parsed.");

    //P2SH redeem script: 0 <sha256 of witness script>
    let witness_script_hash = sha256::Hash::hash(&witness_script.as_bytes());
    let wscript_hash = WScriptHash::from_hash(witness_script_hash);
    let p2sh_redeem_script = Script::new_v0_p2wsh(&wscript_hash);
    println!("P2SH redeem script created.");

    // Create output: 0.001 BTC to the given address
    let output_address = Address::from_str("325UUecEQuyrTd28Xs2hvAxdAjHM7XzqVF").unwrap();
    let output_script_pubkey = output_address.script_pubkey();
    let output_value = Amount::from_sat(100000); // 0.001 BTC = 100,000 satoshis
    let output = TxOut {
        value: output_value.to_sat(),
        script_pubkey: output_script_pubkey,
    };
    println!("Output created: {:?}", output);

    // Create input with manually constructed null outpoint (hash of zeros, index = 0)
    let outpoint = OutPoint {
        txid: Txid::from_slice(&[0u8; 32]).unwrap(),
        vout: 0,
    };

    // Build scriptSig with P2SH redeem script
    let script_sig = bitcoin::blockdata::script::Builder::new()
        .push_slice(p2sh_redeem_script.as_bytes())
        .into_script();

    let input = TxIn {
        previous_output: outpoint,
        script_sig,
        sequence: bitcoin::Sequence(0xffffffff),
        witness: Witness::new(), // Empty witness, populated later
    };
    println!("Input created: {:?}", input);

    // Build transaction: version 2 for SegWit, locktime 0
    let mut tx = Transaction {
        version: 2,
        lock_time: bitcoin::PackedLockTime(0),
        input: vec![input],
        output: vec![output],
    };
    println!("Transaction built: {:?}", tx);

    // Debugging: Print the transaction details before verification
    println!("Transaction details: {:?}", tx);

    // Compute SegWit sighash using BIP143
    let mut cache = SighashCache::new(&tx);
    let sighash_type = EcdsaSighashType::All;
    let sighash = cache
        .segwit_signature_hash(0, &witness_script, output_value.to_sat(), sighash_type)
        .expect("Failed to compute sighash");
    println!("Sighash computed: {:x}", sighash);

    // Sign the sighash with both private keys
    let message = Message::from_slice(&sighash[..]).unwrap();
    let sig1 = secp.sign_ecdsa(&message, &privkey1.inner);
    let sig2 = secp.sign_ecdsa(&message, &privkey2.inner);
    println!("Signatures created.");

    // Serialize signatures with sighash type (SIGHASH_ALL)
    let mut sig1_bytes = sig1.serialize_der().to_vec();
    sig1_bytes.push(sighash_type.to_u32() as u8);
    
    let mut sig2_bytes = sig2.serialize_der().to_vec();
    sig2_bytes.push(sighash_type.to_u32() as u8);

    // Set witness stack with swapped signature order
    let mut witness = Witness::new();
    witness.push(&[]); // Required dummy element for multisig
    witness.push(&sig2_bytes); // Changed order: sig2 first
    witness.push(&sig1_bytes); // Changed order: sig1 second
    witness.push(&witness_script[..]);
    tx.input[0].witness = witness;
    println!("Witness stack set.");

    // Debugging: Print the witness stack details
    println!("Witness stack elements: {}", tx.input[0].witness.len());
    for (i, item) in tx.input[0].witness.iter().enumerate() {
        println!("Witness item {}: {:?}", i, item);
    }

    // Serialize the transaction to hex
    let tx_hex = serialize_hex(&tx);
    println!("Transaction serialized: {}...", &tx_hex[..20]);

    // Write the serialized transaction to out.txt
    fs::write("out.txt", &tx_hex).expect("Failed to write transaction hex to out.txt");
    println!("Transaction hex written to out.txt");

    // Debugging: Print the full transaction hex
    println!("Full transaction hex: {}", tx_hex);
}