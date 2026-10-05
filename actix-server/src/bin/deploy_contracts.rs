// SPDX-License-Identifier: APACHE-2.0

//! Deploys the Asset and AssetFactory contracts to a fresh dev chain, such as the
//! Anvil node in docker-compose.yml. Does nothing if the factory is already there.

use std::env;
use std::sync::Arc;

use anyhow::{ensure, Context, Result};
use ethers::{middleware::{Middleware, SignerMiddleware}, providers::{Http, Provider}, signers::{LocalWallet, Signer}, types::Address};
use trust_server::contracts::{asset::Asset, assetfactory::AssetFactory};

fn var(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("${name} must be set."))
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let factory_addr: Address = var("ASSET_FACTORY_ADDR")?.parse()?;
    let chain_id = var("CHAIN_ID")?.parse::<u64>()?;
    let deployer = var("DEPLOYER_PRIVATE_KEY")?.parse::<LocalWallet>()?.with_chain_id(chain_id);
    let provider = Provider::<Http>::try_from(var("RPC_PROVIDER")?)?;
    let client = Arc::new(SignerMiddleware::new(provider, deployer));

    if !client.get_code(factory_addr, None).await?.is_empty() {
        println!("Contracts already deployed, AssetFactory at {factory_addr:?}");
        return Ok(());
    }

    let asset = Asset::deploy(client.clone(), ())?.send().await?;
    println!("Asset deployed at {:?}", asset.address());
    let factory = AssetFactory::deploy(client, asset.address())?.send().await?;
    println!("AssetFactory deployed at {:?}", factory.address());

    ensure!(
        factory.address() == factory_addr,
        "AssetFactory was deployed at {:?}, but ASSET_FACTORY_ADDR is {factory_addr:?}",
        factory.address()
    );
    Ok(())
}
