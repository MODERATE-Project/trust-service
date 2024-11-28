use std::env;
use std::fs::File;
use actix_web::{get, post};
use actix_web::{web, HttpResponse, Error};
use ipfs_api_backend_actix::{IpfsClient, TryFromUri};
use crate::errors::TrustServiceError;
use crate::models::log_model::Log;
use crate::services::mongodb_repo::MongoRepo;
use crate::services::ipfs::IpfsService;


/// This function takes the log file,
/// then it publishes it to IPFS.
/// It gets the CID from IPFS, then stores the CID in the DB.
/// When storing the CID it updates the document in the DB with the same name or create it.
pub async fn publish_log_internal(mongodb_repo: &MongoRepo) -> Result<(), TrustServiceError> {
    log::info!("Publishing log to IPFS");
    let filename = env::var("LOG_FILE_NAME").expect("$LOG_FILE_NAME must be set.");

    let file = File::open(filename.clone()).map_err(|e| TrustServiceError::FileOpenError)?;

    let ipfs_client = IpfsService::new();

    // Upload the file on IPFS
    let cid = ipfs_client.add_file(filename.as_str()).await?;
    log::info!("Log added to IPFS with: {cid}");
    let log_doc = Log {
        name: filename,
        cid: cid,
    };
    log::info!("Storing CID in the DB");
    mongodb_repo.store_log_cid(log_doc).await?;

    drop(ipfs_client);
    Ok(())
}

/// This API call the publish_log_internal to push the log to IPFS

#[post("")]
async fn publish_log(mongodb_repo: web::Data<MongoRepo>) -> Result<HttpResponse, TrustServiceError> {
    publish_log_internal(&mongodb_repo).await?;
    Ok(HttpResponse::Ok().body("File uploaded successfully"))
}


/// This API retrieves the log file from IPFS
/// It reads the CID from the DB
/// The document that contains the CID has a fixed field called name.
/// So having that name allows to find the CID
#[get("")]
async fn get_log(mongodb_repo: web::Data<MongoRepo>) -> Result<HttpResponse, Error> {

    // get the CID from the DB
    let cid = mongodb_repo.get_log_cid().await?;
    log::info!("Retrieving from IPFS");
    let ipfs_client = IpfsService::new();

    // retrieve from IPFS
    let file = ipfs_client.get_file(cid.as_str()).await;
    
    drop(ipfs_client);
    
    match file {
        Ok(data) => Ok(HttpResponse::Ok().body(data)),
        Err(e) => Ok(HttpResponse::InternalServerError().body(format!("Error: {}", e))),
    }
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        // prefixes all resources and routes attached to it...
        web::scope("/log")
            .service(publish_log)
            .service(get_log)
    );
}