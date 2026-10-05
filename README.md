# Trust service

This project is a microservice for creating and verifying proofs of data authenticity and data integrity. The application stores the proofs on the [IOTA Tangle](https://wiki.iota.org/shimmer/). To answer the requirement that a user should not own crypto tokens, a centralized approach has been used where the service handles the identity keys of the user. The microservice also exposes the API to mint an NFT representing a dataset.
The microservice logs are stored on IPFS, and the CID to retrieve them is stored in traditional storage.

## Getting started  

### Requirements

- `Rust` and `Cargo`. Follow the [documentation](https://doc.rust-lang.org/cargo/getting-started/installation.html) to install them. 
- `Docker` and `Docker compose`


## Run

Beware of the configuration of the environment variables.
Note: Modify `.env` and `.mongo.env` reasonably. (`ADDR`, `MONGO_ENDPOINT`, `ASSET_FACTORY_ADDR`,`L2_PRIVATE_KEY`)

### Locally

For testing the application with MongoDB, follow these steps:
- Run `docker compose --profile dev up -d` to start MongoDB, IPFS and the [local chain](#local-chain).
- Create a database called `MODERATE`.
- Create a collection called `Users`.
- Use [MongoDB Compass](https://www.mongodb.com/products/compass) to view the database content.
Note: MongoDB Compass is a tool that can be used to interact with MongoDB databases and inspect their content.

Create the smart contract Rust bindings (mandatory the first time or if the smart contracts change)
```shell
cd abigen
# assuming the ipr-management folder is located in the same root folder of the trust-service
cargo run -- --contract AssetFactory --abi-source "../../ipr-management/artifacts/contracts/AssetFactory.sol/AssetFactory.json"
cargo run -- --contract Asset --abi-source "../../ipr-management/artifacts/contracts/Asset.sol/Asset.json"
```

Deploy smart contracts to the local chain, then start the application:
```shell
cd actix-server
cargo run --release --bin deploy-contracts
cargo run --release --bin actix-trust-service
```

### Via docker

Copy the smart contract json files to create the Rust bindings (mandatory the first time or if the smart contracts change)
```shell
    mkdir smart-contracts
    # assuming the ipr-management folder is located in the same root folder of the trust-service
    cp ../ipr-management/artifacts/contracts/AssetFactory.sol/AssetFactory.json ./smart-contracts
    cp ../ipr-management/artifacts/contracts/Asset.sol/Asset.json ./smart-contracts

```

Commands for building the app’s container image and starting the app container:
```shell
docker compose --profile deploy up -d
```

Compose passes `actix-server/.env` and `actix-server/.mongo.env` to the container and keeps the service state in the `trust_data` volume. The `deploy` profile also starts the local chain and deploys the smart contracts before the service starts.

### Local chain

Both profiles run a private chain instead of connecting to a public network:

- `iota` runs the archived [`hornet-nest`](https://github.com/iotaledger/hornet/tree/v2.0.1/hornet-nest) image: a private IOTA Stardust network with two Hornet nodes, a coordinator, an indexer, a faucet and a dashboard at <http://localhost:8082>. The faucet holds the whole token supply, and the service requests funds from it as it needs them. DIDs use the `tst` network, as in `did:iota:tst:0x…`.
- `evm` runs an [Anvil](https://getfoundry.sh/anvil/reference/anvil) node with chain ID 1074. `actix-server/.env` uses two of its prefunded dev accounts: account 1 signs the service's transactions and account 0 deploys the contracts. On a fresh chain, `deploy-contracts` always puts `AssetFactory` at the `ASSET_FACTORY_ADDR` in `.env`.

This chain is for local testing only. Keys are public and accessible only from localhost. To fully reset all related data (chain, wallet, MongoDB), run `docker compose --profile deploy down -v` and delete `mongodb_data_container/`.

## Container images

The `docker-publish.yml` workflow builds the image and publishes it as a public package on the GitHub Container Registry:

* `ghcr.io/moderate-project/trust-service`

| Event                       | Tags                                         |
| --------------------------- | -------------------------------------------- |
| Push to `main`              | `main`, `latest`, `sha-<short-sha>`          |
| Release tag (e.g. `v0.1.1`) | `0.1.1`, `sha-<short-sha>`                   |
| Pull request to `main`      | Built to validate the Dockerfile, not pushed |

No credentials are needed to pull it:

```console
docker pull ghcr.io/moderate-project/trust-service:latest
```

### Configuration

The image contains no credentials or network settings. Pass them as environment variables. At startup, the service exits with an error naming each of these that is unset or empty:

| Group        | Variables                                                                                                                                                                  |
| ------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Credentials  | `L2_PRIVATE_KEY`, `MNEMONIC`, `STRONGHOLD_PASSWORD`, `KEY_STORAGE_MNEMONIC`, `KEY_STORAGE_STRONGHOLD_PASSWORD`, `MONGO_INITDB_ROOT_USERNAME`, `MONGO_INITDB_ROOT_PASSWORD` |
| IOTA and EVM | `NODE_URL`, `FAUCET_URL`, `EXPLORER_URL`, `RPC_PROVIDER`, `CHAIN_ID`, `ASSET_FACTORY_ADDR`                                                                                 |
| MongoDB      | `MONGO_ENDPOINT_D`, `MONGO_DATABASE`                                                                                                                                       |

`actix-server/.env` has example values for the [local chain](#local-chain). Each installation needs its own `MNEMONIC` and `KEY_STORAGE_MNEMONIC`. Run this once for each:

```console
docker run --rm --entrypoint gen-mnemonic ghcr.io/moderate-project/trust-service:latest
```

The image sets defaults for the listen address, port, log level and state paths. Mount a volume at `/var/lib/trust`, which holds the wallet, the DID key storage and the access log. The service signs proofs with keys that exist only in the key storage, so if you lose it, you can no longer create proofs for existing DIDs.

The service connects to IPFS at `ipfs:5001`, so the IPFS container must be reachable as `ipfs`.

## Usage

<!-- Provide instructions and examples for use. Include screenshots as needed. -->
- [API Reference](./actix-server/api/specifications.yaml)
- [Postman Collection](./actix-server/api/Trust-service.postman_collection.json)


## License

[Apache-2.0](http://www.apache.org/licenses/LICENSE-2.0)