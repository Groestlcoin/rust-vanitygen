extern crate groestlcoin;

use groestlcoin::hashes::Hash;
use groestlcoin::hashes::HashEngine;
use groestlcoin::network::constants::Network;
use groestlcoin::schnorr::PublicKey;
use groestlcoin::secp256k1::rand::rngs::OsRng;
use groestlcoin::secp256k1::Secp256k1;
use groestlcoin::util::address::Address;
use groestlcoin::util::taproot::TapTweakHash;
use groestlcoin::util::ecdsa::PrivateKey;

use std::env;

fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        println!("Invalid number of args");
        return;
    }

    let prefix = args[1].to_lowercase();
    if prefix.len() <= 5 {
        println!("Prefix is too short");
        return;
    }
    if prefix.get(0..5) != Some("grs1p") {
        println!("Invalid prefix, must begin with grs1p");
        return;
    }

    const CHARSET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let prefix_split: Vec<&str> = prefix.split("1").collect();
    for pc in prefix_split[1].chars() {
        if !CHARSET.contains(pc) {
            println!("Invalid character in prefix");
            return;
        }
    }

    let mut merkle_root: Vec<u8> = Vec::new();
    if args.len() == 3 {
        merkle_root = hex::decode(&args[2]).unwrap();
    }

    let secp = Secp256k1::new();

    let mut rng = OsRng::new().unwrap();

    loop {
        let (internal_seckey, internal_pubkey) = secp.generate_schnorrsig_keypair(&mut rng);

        let mut tweak: Vec<u8> = Vec::new();
        tweak.extend_from_slice(&internal_pubkey.serialize());
        tweak.extend_from_slice(&merkle_root);
        let mut engine = TapTweakHash::engine();
        engine.input(&tweak);
        let tweak_value: [u8; 32] = TapTweakHash::from_engine(engine).into_inner();

        let mut output_seckey = internal_seckey.clone();
        output_seckey.tweak_add_assign(&secp, &tweak_value).unwrap();

        let output_pubkey = PublicKey::from_keypair(&secp, &output_seckey);

        let addr = Address::p2tr(output_pubkey, Network::Groestlcoin);

        if addr.to_string().get(0..prefix.len()) == Some(&prefix) {
            let internal_privkey = PrivateKey::from_slice(&internal_seckey.serialize_secret(), Network::Groestlcoin).unwrap();
            println!("internal_privkey: {}", internal_privkey.to_wif());
            println!("internal_pubkey: {}", internal_pubkey);
            println!("output_pubkey: {}", output_pubkey);
            println!("Address: {}", addr);
            break;
        }
    }
}
