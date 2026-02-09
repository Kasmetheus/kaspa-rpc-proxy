use serde::{Deserialize, Serialize};

/// Request to get a block by hash
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBlockRequest {
    pub hash: String,
    #[serde(default = "default_true")]
    pub include_transactions: bool,
}

fn default_true() -> bool {
    true
}

/// Request to submit a transaction
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitTransactionRequest {
    pub transaction: TransactionInput,
    #[serde(default)]
    pub allow_orphan: bool,
}

/// Simplified transaction input format
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInput {
    pub version: Option<u32>,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: Option<u64>,
    pub subnetwork_id: Option<String>,
    pub gas: Option<u64>,
    pub payload: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxInput {
    pub previous_outpoint: Outpoint,
    pub signature_script: String,
    pub sequence: u64,
    pub sig_op_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Outpoint {
    pub transaction_id: String,
    pub index: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxOutput {
    pub amount: u64,
    pub script_public_key: ScriptPublicKey,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptPublicKey {
    pub script_public_key: String,
    pub version: u16,
}

/// Request for DAG tips
#[derive(Debug, Deserialize)]
pub struct GetDAGTipsRequest {}

/// Request to subscribe to UTXO changes
#[derive(Debug, Deserialize)]
pub struct SubscribeUTXORequest {
    pub addresses: Vec<String>,
}

/// Generic RPC response wrapper
#[derive(Debug, Serialize)]
pub struct RpcResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub latency_ms: f64,
}

impl<T> RpcResponse<T> {
    pub fn success(data: T, latency_ms: f64) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            latency_ms,
        }
    }

    pub fn error(error: String, latency_ms: f64) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            latency_ms,
        }
    }
}

/// Block response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockResponse {
    pub hash: String,
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub verbose_data: Option<BlockVerboseData>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeader {
    pub version: u32,
    pub hash_merkle_root: String,
    pub accepted_id_merkle_root: String,
    pub utxo_commitment: String,
    pub timestamp: i64,
    pub bits: u32,
    pub nonce: u64,
    pub daa_score: u64,
    pub blue_work: String,
    pub blue_score: u64,
    pub pruning_point: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub transaction_id: String,
    pub hash: String,
    pub mass: u64,
    pub inputs: Vec<TransactionInputVerbose>,
    pub outputs: Vec<TransactionOutput>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInputVerbose {
    pub previous_outpoint: OutpointVerbose,
    pub signature_script: String,
    pub sequence: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutpointVerbose {
    pub transaction_id: String,
    pub index: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOutput {
    pub amount: u64,
    pub script_public_key: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockVerboseData {
    pub hash: String,
    pub difficulty: f64,
    pub selected_parent_hash: String,
    pub transaction_ids: Vec<String>,
    pub is_header_only: bool,
    pub blue_score: u64,
    pub is_chain_block: bool,
}

/// Submit transaction response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitTransactionResponse {
    pub transaction_id: String,
}

/// DAG tips response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DAGTipsResponse {
    pub tip_hashes: Vec<String>,
    pub block_count: u64,
    pub header_count: u64,
    pub difficulty: f64,
    pub past_median_time: i64,
    pub virtual_parent_hashes: Vec<String>,
    pub pruning_point_hash: String,
    pub virtual_daa_score: u64,
}
