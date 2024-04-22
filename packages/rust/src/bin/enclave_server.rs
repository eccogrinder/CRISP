mod util;

use std::{env, error::Error, process::exit, sync::Arc, fs, path::Path, str};
use chrono::{DateTime, TimeZone, Utc};
use console::style;
use fhe::{
    bfv::{BfvParametersBuilder, Ciphertext, Encoding, Plaintext, PublicKey, SecretKey},
    mbfv::{AggregateIter, CommonRandomPoly, DecryptionShare, PublicKeyShare},
};
use fhe_traits::{FheDecoder, FheEncoder, FheEncrypter, Serialize as FheSerialize}; // TODO: see if we can use serde Serialize in fhe lib
use rand::{distributions::Uniform, prelude::Distribution, rngs::OsRng, thread_rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use util::timeit::{timeit, timeit_n};
use serde::{Deserialize, Serialize};
//use serde_json::{Result, Value};

use iron::prelude::*;
use iron::status;
use iron::mime::Mime;
use router::Router;
use std::io::Read;
use std::fs::File;

use walkdir::WalkDir;

use ethers::{
    prelude::{abigen, Abigen},
    providers::{Http, Provider},
    middleware::SignerMiddleware,
    signers::{LocalWallet, Signer, Wallet},
    types::{Address, U256, Bytes, TxHash},
    core::k256,
    utils,
};

use sled::Db;

// pick a string at random
fn pick_response() -> String {
    "Test".to_string()
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonResponse {
    response: String
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonResponseTxHash {
    response: String,
    tx_hash: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonRequest {
    response: String,
    pk_share: Vec<u8>,
    id: u32,
    round_id: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct CrispConfig {
    round_id: u32,
    chain_id: u32,
    voting_address: String,
    ciphernode_count: u32,
    voter_count: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct RoundCount {
    round_count: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct PKShareCount {
    round_id: u32,
    share_id: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct PKRequest {
    round_id: u32,
    pk_bytes: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CRPRequest {
    round_id: u32,
    crp_bytes: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TimestampRequest {
    round_id: u32,
    timestamp: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct VoteCountRequest {
    round_id: u32,
    vote_count: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct SKSShareRequest {
    response: String,
    sks_share: Vec<u8>,
    id: u32,
    round_id: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct EncryptedVote {
    round_id: u32,
    enc_vote_bytes: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GetRoundRequest {
    round_id: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct SKSSharePoll {
    response: String,
    round_id: u32,
    ciphernode_count: u32, //TODO: dont need this
}

#[derive(Debug, Deserialize, Serialize)]
struct SKSShareResponse {
    response: String,
    round_id: u32,
    sks_shares: Vec<Vec<u8>>,
}

// fn register_cyphernode(req: &mut Request) -> IronResult<Response> {
    // register ip address or some way to contact nodes when a computation request comes in

// }

#[derive(Debug, Deserialize, Serialize)]
struct Round {
    id: u32,
    voting_address: String,
    chain_id: u32,
    ciphernode_count: u32,
    pk_share_count: u32,
    sks_share_count: u32,
    vote_count: u32,
    crp: Vec<u8>,
    pk: Vec<u8>,
    start_time: i64,
    ciphernode_total:  u32,
    ciphernodes: Vec<Ciphernode>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Ciphernode {
    id: u32,
    pk_share: Vec<u8>,
    sks_share: Vec<u8>,
}

fn get_state(round_id: u32) -> (Round, Db) {
    let pathdb = env::current_dir().unwrap();
    let mut pathdbst = pathdb.display().to_string();
    pathdbst.push_str("/database");
    let db = sled::open(pathdbst.clone()).unwrap();
    let mut round_key = round_id.to_string();
    round_key.push_str("-storage");
    println!("Database key is {:?}", round_key);
    let state_out = db.get(round_key).unwrap().unwrap();
    let state_out_str = str::from_utf8(&state_out).unwrap();
    let state_out_struct: Round = serde_json::from_str(&state_out_str).unwrap();
    (state_out_struct, db)
}

#[tokio::main]
async fn broadcast_enc_vote(req: &mut Request) -> IronResult<Response> {
    let mut payload = String::new();
    // read the POST body
    req.body.read_to_string(&mut payload).unwrap();
    let mut incoming: EncryptedVote = serde_json::from_str(&payload).unwrap();

    let (mut state, db) = get_state(incoming.round_id);
    state.vote_count = state.vote_count + 1;
    let state_str = serde_json::to_string(&state).unwrap();
    let state_bytes = state_str.into_bytes();
    let key = incoming.round_id.to_string();
    db.insert(key, state_bytes).unwrap();

    let sol_vote = Bytes::from(incoming.enc_vote_bytes);
    let tx_hash = call_contract(sol_vote, state.voting_address).await.unwrap();
    let mut converter = "0x".to_string();
    for i in 0..32 {
        if(tx_hash[i] <= 16) {
            converter.push_str("0");
            converter.push_str(&format!("{:x}", tx_hash[i]));
        } else {
            converter.push_str(&format!("{:x}", tx_hash[i]));
        }
    }

    let response = JsonResponseTxHash { response: "tx_sent".to_string(), tx_hash: converter };
    let out = serde_json::to_string(&response).unwrap();

    let content_type = "application/json".parse::<Mime>().unwrap();
    println!("Request for round {:?} send vote tx", incoming.round_id);
    Ok(Response::with((content_type, status::Ok, out)))
}

async fn call_contract(enc_vote: Bytes, address: String) -> Result<TxHash, Box<dyn std::error::Error + Send + Sync>> {
    println!("calling voting contract");

    let infura_key = "INFURAKEY";
    let infura_val = env::var(infura_key).unwrap();
    let mut RPC_URL = "https://sepolia.infura.io/v3/".to_string();
    RPC_URL.push_str(&infura_val);

    let provider = Provider::<Http>::try_from(RPC_URL.clone())?;
    // let block_number: U64 = provider.get_block_number().await?;
    // println!("{block_number}");
    abigen!(
        IVOTE,
        r#"[
            function voteEncrypted(bytes memory _encVote) public
            function getVote(address id) public returns(bytes memory)
            event Transfer(address indexed from, address indexed to, uint256 value)
        ]"#,
    );

    //const RPC_URL: &str = "https://eth.llamarpc.com";
    let VOTE_ADDRESS: &str = &address;

    let eth_key = "PRIVATEKEY";
    let eth_val = env::var(eth_key).unwrap();
    let wallet: LocalWallet = eth_val
        .parse::<LocalWallet>().unwrap()
        .with_chain_id(11155111 as u64);

    // 6. Wrap the provider and wallet together to create a signer client
    let client = SignerMiddleware::new(provider.clone(), wallet.clone());
    //let client = Arc::new(provider);
    let address: Address = VOTE_ADDRESS.parse()?;
    let contract = IVOTE::new(address, Arc::new(client.clone()));

    let test = contract.vote_encrypted(enc_vote).send().await?.clone();
    println!("{:?}", test);
    Ok(test)
}

fn get_round_state(req: &mut Request) -> IronResult<Response> {
    let mut payload = String::new();
    // read the POST body
    req.body.read_to_string(&mut payload).unwrap();
    let mut incoming: GetRoundRequest = serde_json::from_str(&payload).unwrap();
    println!("Request config for round {:?}", incoming.round_id);

    let (state, db) = get_state(incoming.round_id);
    let out = serde_json::to_string(&state).unwrap();

    let content_type = "application/json".parse::<Mime>().unwrap();
    Ok(Response::with((content_type, status::Ok, out)))
}

fn get_vote_count_by_round(req: &mut Request) -> IronResult<Response> {
    let mut payload = String::new();
    // read the POST body
    req.body.read_to_string(&mut payload).unwrap();
    let mut incoming: VoteCountRequest = serde_json::from_str(&payload).unwrap();
    println!("Request for round {:?} crp", incoming.round_id);

    let (state, db) = get_state(incoming.round_id);
    incoming.vote_count = state.vote_count;
    let out = serde_json::to_string(&incoming).unwrap();

    let content_type = "application/json".parse::<Mime>().unwrap();
    Ok(Response::with((content_type, status::Ok, out)))
}

fn get_start_time_by_round(req: &mut Request) -> IronResult<Response> {
    let mut payload = String::new();
    // read the POST body
    req.body.read_to_string(&mut payload).unwrap();
    let mut incoming: TimestampRequest = serde_json::from_str(&payload).unwrap();
    println!("Request for round {:?} crp", incoming.round_id);

    let (state, db) = get_state(incoming.round_id);
    incoming.timestamp = state.start_time;
    let out = serde_json::to_string(&incoming).unwrap();

    let content_type = "application/json".parse::<Mime>().unwrap();
    Ok(Response::with((content_type, status::Ok, out)))
}

fn get_crp_by_round(req: &mut Request) -> IronResult<Response> {
    let mut payload = String::new();
    // read the POST body
    req.body.read_to_string(&mut payload).unwrap();
    let mut incoming: CRPRequest = serde_json::from_str(&payload).unwrap();
    println!("Request for round {:?} crp", incoming.round_id);

    let (state, db) = get_state(incoming.round_id);
    incoming.crp_bytes = state.crp;
    let out = serde_json::to_string(&incoming).unwrap();

    let content_type = "application/json".parse::<Mime>().unwrap();
    Ok(Response::with((content_type, status::Ok, out)))
}

fn get_pk_by_round(req: &mut Request) -> IronResult<Response> {
    let mut payload = String::new();
    // read the POST body
    req.body.read_to_string(&mut payload).unwrap();
    let mut incoming: PKRequest = serde_json::from_str(&payload).unwrap();

    let (state, db) = get_state(incoming.round_id);
    incoming.pk_bytes = state.pk;
    let out = serde_json::to_string(&incoming).unwrap();

    let content_type = "application/json".parse::<Mime>().unwrap();
    println!("Request for round {:?} public key", incoming.round_id);
    Ok(Response::with((content_type, status::Ok, out)))
}

fn get_pk_share_count(req: &mut Request) -> IronResult<Response> {
    let mut payload = String::new();
    // read the POST body
    req.body.read_to_string(&mut payload).unwrap();

    let mut incoming: PKShareCount = serde_json::from_str(&payload).unwrap();

    let (state, db) = get_state(incoming.round_id);
    incoming.share_id = state.pk_share_count;
    let out = serde_json::to_string(&incoming).unwrap();

    let content_type = "application/json".parse::<Mime>().unwrap();
    Ok(Response::with((content_type, status::Ok, out)))
}

fn get_rounds(req: &mut Request) -> IronResult<Response> {
    let pathdb = env::current_dir().unwrap();
    let mut pathdbst = pathdb.display().to_string();
    pathdbst.push_str("/database");
    let db = sled::open(pathdbst.clone()).unwrap();
    let key = "round_count";
    let mut round = db.get(key).unwrap();
    if(round == None) {
        println!("initializing first round in db");
        db.insert(key, b"0".to_vec()).unwrap();
        round = db.get(key).unwrap();
    }
    let mut round_key = std::str::from_utf8(round.unwrap().as_ref()).unwrap().to_string();
    let mut round_int = round_key.parse::<u32>().unwrap();

    let count = RoundCount {round_count: round_int};
    println!("round_count: {:?}", count.round_count);

    let response = JsonResponse { response: "Round Count Retrieved".to_string() };
    let out = serde_json::to_string(&count).unwrap();
    println!("get rounds hit");

    let content_type = "application/json".parse::<Mime>().unwrap();
    Ok(Response::with((content_type, status::Ok, out)))
}

fn init_crisp_round(req: &mut Request) -> IronResult<Response> {
    println!("generating round crp");

    let degree = 4096;
    let plaintext_modulus: u64 = 4096;
    let moduli = vec![0xffffee001, 0xffffc4001, 0x1ffffe0001];

    // Let's generate the BFV parameters structure.
    let params = timeit!(
        "Parameters generation",
        BfvParametersBuilder::new()
            .set_degree(degree)
            .set_plaintext_modulus(plaintext_modulus)
            .set_moduli(&moduli)
            .build_arc().unwrap()
    );
    let crp = CommonRandomPoly::new(&params, &mut thread_rng()).unwrap();
    let crp_bytes = crp.to_bytes();

    let mut payload = String::new();

    // read the POST body
    req.body.read_to_string(&mut payload).unwrap();

    // we're expecting the POST to match the format of our JsonRequest struct
    let incoming: CrispConfig = serde_json::from_str(&payload).unwrap();
    println!("ID: {:?}", incoming.round_id); // TODO: check that client sent the expected next round_id
    println!("Address: {:?}", incoming.voting_address);

    // --------------
    let pathdb = env::current_dir().unwrap();
    let mut pathdbst = pathdb.display().to_string();
    pathdbst.push_str("/database");

    let db = sled::open(pathdbst.clone()).unwrap();
    let key = "round_count";
    //db.remove(key)?;
    let round = db.get(key).unwrap();
    if(round == None) {
        println!("initializing first round in db");
        db.insert(key, b"0".to_vec()).unwrap();
    }
    let mut round_key = std::str::from_utf8(round.unwrap().as_ref()).unwrap().to_string();
    let mut round_int = round_key.parse::<u32>().unwrap();
    round_int = round_int + 1;
    let mut inc_round_key = round_int.to_string();
    inc_round_key.push_str("-storage");
    println!("Database key is {:?} and round int is {:?}", inc_round_key, round_int);

    let init_time = Utc::now();
    let timestamp = init_time.timestamp();
    println!("timestamp {:?}", timestamp);

    let state = Round {
        id: round_int,
        voting_address: incoming.voting_address,
        chain_id: incoming.chain_id,
        ciphernode_count: 0,
        pk_share_count: 0,
        sks_share_count: 0,
        vote_count: 0,
        crp: crp_bytes,
        pk: vec![0],
        start_time: timestamp,
        ciphernode_total: incoming.ciphernode_count,
        ciphernodes: vec![
            Ciphernode {
                id: 0,
                pk_share: vec![0],
                sks_share: vec![0],
            }
        ],
    };

    let state_str = serde_json::to_string(&state).unwrap();
    let state_bytes = state_str.into_bytes();
    let key2 = round_int.to_string();
    db.insert(inc_round_key, state_bytes).unwrap();

    let new_round_bytes = key2.into_bytes();
    db.insert(key, new_round_bytes).unwrap();

    // create a response with our random string, and pass in the string from the POST body
    let response = JsonResponse { response: "CRISP Initiated".to_string() };
    let out = serde_json::to_string(&response).unwrap();

    let content_type = "application/json".parse::<Mime>().unwrap();
    Ok(Response::with((content_type, status::Ok, out)))
}


async fn aggregate_pk_shares(round_id: u32, db: &Db) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("aggregating validator keyshare");

    let degree = 4096;
    let plaintext_modulus: u64 = 4096;
    let moduli = vec![0xffffee001, 0xffffc4001, 0x1ffffe0001];

    // Generate a deterministic seed for the Common Poly
    //let mut seed = <ChaCha8Rng as SeedableRng>::Seed::default();

    // Let's generate the BFV parameters structure.
    let params = timeit!(
        "Parameters generation",
        BfvParametersBuilder::new()
            .set_degree(degree)
            .set_plaintext_modulus(plaintext_modulus)
            .set_moduli(&moduli)
            .build_arc()?
    );

    let mut round_key = round_id.to_string();
    round_key.push_str("-storage");
    println!("Database key is {:?}", round_key);

    let state_out = db.get(round_key.clone()).unwrap().unwrap();
    let state_out_str = str::from_utf8(&state_out).unwrap();
    let mut state: Round = serde_json::from_str(&state_out_str).unwrap();
    println!("checking db after drop {:?}", state.ciphernode_count);
    println!("{:?}", state.ciphernodes[0].id);
    //println!("{:?}", state.ciphernodes[0].pk_share);

    //let crp = CommonRandomPoly::new_deterministic(&params, seed)?;
    let crp = CommonRandomPoly::deserialize(&state.crp, &params)?;

    // Party setup: each party generates a secret key and shares of a collective
    // public key.
    struct Party {
        pk_share: PublicKeyShare,
    }

    let mut parties :Vec<Party> = Vec::new();
    for i in 1..state.ciphernode_total + 1 { // todo fix init code that causes offset
        // read in pk_shares from storage
        println!("Aggregating PKShare... id {}", i);
        let data_des = PublicKeyShare::deserialize(&state.ciphernodes[i as usize].pk_share, &params, crp.clone()).unwrap();
        // let pk_share = PublicKeyShare::new(&sk_share, crp.clone(), &mut thread_rng())?;
        parties.push(Party { pk_share: data_des });
    }

    // Aggregation: this could be one of the parties or a separate entity. Or the
    // parties can aggregate cooperatively, in a tree-like fashion.
    let pk = timeit!("Public key aggregation", {
        let pk: PublicKey = parties.iter().map(|p| p.pk_share.clone()).aggregate()?;
        pk
    });
    //println!("{:?}", pk);
    println!("Multiparty Public Key Generated");
    let store_pk = pk.to_bytes();
    state.pk = store_pk;
    let state_str = serde_json::to_string(&state).unwrap();
    let state_bytes = state_str.into_bytes();
    db.insert(round_key, state_bytes).unwrap();
    println!("aggregate pk stored for round {:?}", round_id);
    Ok(())
}

fn handler(req: &mut Request) -> IronResult<Response> {
    let response = JsonResponse { response: pick_response() };
    let out = serde_json::to_string(&response).unwrap();

    let content_type = "application/json".parse::<Mime>().unwrap();
    Ok(Response::with((content_type, status::Ok, out)))
}

// polling endpoint for sks shares

fn register_sks_share(req: &mut Request) -> IronResult<Response> {
    let mut payload = String::new();

    // read the POST body
    req.body.read_to_string(&mut payload).unwrap();

    // we're expecting the POST to match the format of our JsonRequest struct
    let incoming: SKSShareRequest = serde_json::from_str(&payload).unwrap();
    println!("{:?}", incoming.response);
    println!("ID: {:?}", incoming.id); // cipher node id (based on first upload of pk share)
    println!("Round ID: {:?}", incoming.round_id);

    let pathdb = env::current_dir().unwrap();
    let mut pathdbst = pathdb.display().to_string();
    pathdbst.push_str("/database");
    let db = sled::open(pathdbst.clone()).unwrap();

    let mut round_key = incoming.round_id.to_string();
    round_key.push_str("-storage");
    println!("Database key is {:?}", round_key);

    let state_out = db.get(round_key.clone()).unwrap().unwrap();
    let state_out_str = str::from_utf8(&state_out).unwrap();
    let mut state_out_struct: Round = serde_json::from_str(&state_out_str).unwrap();
    state_out_struct.sks_share_count = state_out_struct.sks_share_count + 1;

    let index = incoming.id + 1; // offset from vec push
    state_out_struct.ciphernodes[index as usize].sks_share = incoming.sks_share;
    let state_str = serde_json::to_string(&state_out_struct).unwrap();
    let state_bytes = state_str.into_bytes();
    db.insert(round_key, state_bytes).unwrap();
    println!("sks share stored for node id {:?}", incoming.id);

    // toso get share threshold from client config
    if(state_out_struct.sks_share_count == state_out_struct.ciphernode_total) {
        println!("All sks shares received");
        //aggregate_pk_shares(incoming.round_id).await;
        // TODO: maybe notify cipher nodes
    }

    // create a response with our random string, and pass in the string from the POST body
    let response = JsonResponse { response: pick_response() };
    let out = serde_json::to_string(&response).unwrap();

    let content_type = "application/json".parse::<Mime>().unwrap();
    Ok(Response::with((content_type, status::Ok, out)))
}

fn get_sks_shares(req: &mut Request) -> IronResult<Response> {
    let mut payload = String::new();

    // read the POST body
    req.body.read_to_string(&mut payload).unwrap();

    // we're expecting the POST to match the format of our JsonRequest struct
    let incoming: SKSSharePoll = serde_json::from_str(&payload).unwrap();
    //const length: usize = incoming.cyphernode_count;

    let pathdb = env::current_dir().unwrap();
    let mut pathdbst = pathdb.display().to_string();
    pathdbst.push_str("/database");
    let db = sled::open(pathdbst.clone()).unwrap();

    let mut round_key = incoming.round_id.to_string();
    round_key.push_str("-storage");
    println!("Database key is {:?}", round_key);

    let state_out = db.get(round_key.clone()).unwrap().unwrap();
    let state_out_str = str::from_utf8(&state_out).unwrap();
    let state_out_struct: Round = serde_json::from_str(&state_out_str).unwrap();

    let mut shares = Vec::with_capacity(incoming.ciphernode_count as usize);

    // toso get share threshold from client config
    if(state_out_struct.sks_share_count == state_out_struct.ciphernode_total) {
        println!("All sks shares received... sending to cipher nodes");
        for i in 1..state_out_struct.ciphernode_total + 1 {
            println!("reading share {:?}", i);
            shares.push(state_out_struct.ciphernodes[i as usize].sks_share.clone());
        }
        let response = SKSShareResponse { 
            response: "final".to_string(),
            round_id: incoming.round_id,
            sks_shares: shares,
        };
        let out = serde_json::to_string(&response).unwrap();
        println!("get rounds hit");

        let content_type = "application/json".parse::<Mime>().unwrap();
        Ok(Response::with((content_type, status::Ok, out)))
    } else {
        let response = SKSShareResponse { 
            response: "waiting".to_string(),
            round_id: incoming.round_id,
            sks_shares: shares,
        };
        let out = serde_json::to_string(&response).unwrap();
        println!("get rounds hit");

        let content_type = "application/json".parse::<Mime>().unwrap();
        Ok(Response::with((content_type, status::Ok, out)))
    }
}

#[tokio::main]
async fn register_keyshare(req: &mut Request) -> IronResult<Response> {
    let mut payload = String::new();

    // read the POST body
    req.body.read_to_string(&mut payload).unwrap();

    // we're expecting the POST to match the format of our JsonRequest struct
    let incoming: JsonRequest = serde_json::from_str(&payload).unwrap();
    println!("{:?}", incoming.response);
    println!("ID: {:?}", incoming.id);
    println!("Round ID: {:?}", incoming.round_id);

    let pathdb = env::current_dir().unwrap();
    let mut pathdbst = pathdb.display().to_string();
    pathdbst.push_str("/database");
    let db = sled::open(pathdbst.clone()).unwrap();
    let mut round_key = incoming.round_id.to_string();
    round_key.push_str("-storage");
    println!("Database key is {:?}", round_key);
    let state_out = db.get(round_key.clone()).unwrap().unwrap();
    let state_out_str = str::from_utf8(&state_out).unwrap();
    let mut state: Round = serde_json::from_str(&state_out_str).unwrap();

    state.pk_share_count = state.pk_share_count + 1;
    state.ciphernode_count = state.ciphernode_count + 1;
    let cnode = Ciphernode {
        id: incoming.id,
        pk_share: incoming.pk_share,
        sks_share: vec![0],
    };
    state.ciphernodes.push(cnode);
    let state_str = serde_json::to_string(&state).unwrap();
    let state_bytes = state_str.into_bytes();
    db.insert(round_key, state_bytes).unwrap();

    println!("pk share store for node id {:?}", incoming.id);
    println!("ciphernode count {:?}", state.ciphernode_count);
    println!("ciphernode total {:?}", state.ciphernode_total);
    println!("pk share count {:?}", state.pk_share_count);

    if(state.ciphernode_count == state.ciphernode_total) {
        println!("All shares received");
        aggregate_pk_shares(incoming.round_id, &db).await;
    }

    // create a response with our random string, and pass in the string from the POST body
    let response = JsonResponse { response: pick_response() };
    let out = serde_json::to_string(&response).unwrap();

    let content_type = "application/json".parse::<Mime>().unwrap();
    Ok(Response::with((content_type, status::Ok, out)))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // // use this to reset db
    // let pathdb = env::current_dir().unwrap();
    // let mut pathdbst = pathdb.display().to_string();
    // pathdbst.push_str("/database");
    // let db = sled::open(pathdbst.clone()).unwrap();
    // let key = "round_count";
    // db.remove(key).unwrap();

    // Server Code
    let mut router = Router::new();
    router.get("/", handler, "index");
    router.get("/get_rounds", get_rounds, "get_rounds");
    router.post("/get_pk_share_count", get_pk_share_count, "get_pk_share_count");
    router.post("/register_keyshare", register_keyshare, "register_keyshare");
    router.post("/init_crisp_round", init_crisp_round, "init_crisp_round");
    router.post("/get_pk_by_round", get_pk_by_round, "get_pk_by_round");
    router.post("/register_sks_share", register_sks_share, "register_sks_share");
    router.post("/get_sks_shares", get_sks_shares, "get_sks_shares");
    router.post("/get_crp_by_round", get_crp_by_round, "get_crp_by_round");
    router.post("/broadcast_enc_vote", broadcast_enc_vote, "broadcast_enc_vote");
    router.post("/get_vote_count_by_round", get_vote_count_by_round, "get_vote_count_by_round");
    router.post("/get_start_time_by_round", get_start_time_by_round, "get_start_time_by_round");

    Iron::new(router).http("127.0.0.1:4000").unwrap();

    Ok(())
}
