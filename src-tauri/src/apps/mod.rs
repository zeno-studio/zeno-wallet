pub mod list;


#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode)]
pub struct Apps {
    id: u64,
    name: String,
    app_path: String,
    description: String,
    supported_chain_id_ids: Vec<u64>,
}