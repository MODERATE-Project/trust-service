use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ProofRequestDTO {
    pub asset_hash: String,
    pub metadata_hash: String
}


#[derive(Deserialize, Serialize)]
pub struct ProofResponseDTO {

}
