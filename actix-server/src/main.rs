// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use std::env;
use std::path::Path;
use std::sync::Arc;

use actix_web::{web, App, HttpServer, middleware::Logger};
use anyhow::Context;
use ethers::{middleware::SignerMiddleware, providers::{Http, Provider}, signers::{LocalWallet, Signer}};
use log::log;
use trust_server::{controllers::{did_controller, nft_controller, proof_controller}, services::{iota_state::IotaState, mongodb_repo::MongoRepo}};
use trust_server::controllers::log_controller;

/// Variables the service reads without a fallback. Some are first read after the
/// IOTA node sync or by a request handler, so `main` checks them all before startup.
const REQUIRED_VARS: &[&str] = &[
    "L2_PRIVATE_KEY",
    "MNEMONIC",
    "STRONGHOLD_PASSWORD",
    "KEY_STORAGE_MNEMONIC",
    "KEY_STORAGE_STRONGHOLD_PASSWORD",
    "MONGO_INITDB_ROOT_USERNAME",
    "MONGO_INITDB_ROOT_PASSWORD",
    "NODE_URL",
    "FAUCET_URL",
    "EXPLORER_URL",
    "RPC_PROVIDER",
    "CHAIN_ID",
    "ASSET_FACTORY_ADDR",
    "MONGO_DATABASE",
    "PORT",
    "WALLET_DB_PATH",
    "STRONGHOLD_SNAPSHOT_PATH",
    "KEY_STORAGE_STRONGHOLD_SNAPSHOT_PATH",
    "LOG_FILE_NAME",
];

/// Returns one error naming every required variable that is unset or empty.
fn check_required_vars() -> anyhow::Result<()> {
    let mode_vars = if env::var("RUNNING_IN_DOCKER").is_ok() {
        ["ADDR_D", "MONGO_ENDPOINT_D"]
    } else {
        ["ADDR_L", "MONGO_ENDPOINT_L"]
    };

    let missing: Vec<&str> = REQUIRED_VARS
        .iter()
        .chain(&mode_vars)
        .copied()
        .filter(|name| env::var(name).unwrap_or_default().is_empty())
        .collect();

    anyhow::ensure!(missing.is_empty(), "missing required environment variables: {}", missing.join(", "));
    Ok(())
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {

    // Local runs read these files; containers get their settings from the environment
    for path in [".env", ".mongo.env"] {
        if Path::new(path).exists() {
            dotenv::from_path(path).with_context(|| format!("failed to load {path}"))?;
        }
    }

    env_logger::init();
    check_required_vars()?;

    let mut address = "".to_string();

    if env::var("RUNNING_IN_DOCKER").is_ok(){
        address = std::env::var("ADDR_D").expect("$ADDR must be set.");
        log::info!("Runnig in Docker {}", address);
    } else {
        address = std::env::var("ADDR_L").expect("$ADDR must be set.");
        log::info!("Runnig in Local {}", address);
    };

    //let address = std::env::var("ADDR").expect("$ADDR must be set.");
    let port = std::env::var("PORT").expect("$PORT must be set.").parse::<u16>()?;
    
    let db: MongoRepo = MongoRepo::init().await;
    let db_data: web::Data<MongoRepo> = web::Data::new(db);

    let iota_state: IotaState = IotaState::init().await?;
    let iota_state_data: web::Data<IotaState> = web::Data::new(iota_state);

    // Initialize provider
    let rpc_provider =  std::env::var("RPC_PROVIDER").expect("$RPC_PROVIDER must be set.");
    let chain_id = std::env::var("CHAIN_ID").expect("$CHAIN_ID must be set.");
    // Transactions will be signed with the private key below
    let local_wallet = std::env::var("L2_PRIVATE_KEY").expect("$L2_PRIVATE_KEY must be set")
    .parse::<LocalWallet>()?
    .with_chain_id(chain_id.parse::<u64>().expect("CHAIN_ID is not a number"));


    log::info!("Initializing custom provider");
    let provider = Provider::<Http>::try_from(rpc_provider)?;
    let signer = Arc::new(SignerMiddleware::new(provider, local_wallet));
    let signer_data = web::Data::new(signer);

    log::info!("Starting up on {}:{}", address, port);
    HttpServer::new(move || {
        App::new()
            .app_data(iota_state_data.clone())
            .app_data(db_data.clone())
            .app_data(signer_data.clone())
            .service(web::scope("/api")
                .configure(did_controller::scoped_config)
                .configure(proof_controller::scoped_config)
                .configure(nft_controller::scoped_config)
                .configure(log_controller::scoped_config)
            )
            .wrap(Logger::default())
    })
    .bind((address, port))?
    .run()
    .await.map_err(anyhow::Error::from)
}