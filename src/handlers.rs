use crate::{auth, client, error::RpcError, metrics, models::*, AppState};
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::time::Instant;

/// Health check endpoint
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// Metrics endpoint (Prometheus format)
pub async fn metrics() -> String {
    metrics::export_metrics()
}

/// Get block by hash
pub async fn get_block(
    State(state): State<AppState>,
    // Optional JWT auth middleware can be added here
    Json(request): Json<GetBlockRequest>,
) -> Result<Json<RpcResponse<BlockResponse>>, RpcError> {
    let start = Instant::now();
    
    // Validate hash format
    if !is_valid_hash(&request.hash) {
        return Err(RpcError::BadRequest("Invalid block hash format".into()));
    }

    // Call Kaspa node
    let response = state
        .kaspa_client
        .get_block(request.hash.clone(), request.include_transactions)
        .await?;

    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
    metrics::record_latency("get_block", latency_ms);

    // Convert proto response to JSON model
    let block = response.block.ok_or_else(|| {
        RpcError::InvalidResponse("Block data missing".into())
    })?;

    let header = block.header.as_ref().ok_or_else(|| {
        RpcError::InvalidResponse("Block header missing".into())
    })?;

    let block_response = BlockResponse {
        hash: header.hash.clone(),
        header: BlockHeader {
            version: header.version,
            hash_merkle_root: header.hash_merkle_root.clone(),
            accepted_id_merkle_root: header.accepted_id_merkle_root.clone(),
            utxo_commitment: header.utxo_commitment.clone(),
            timestamp: header.timestamp,
            bits: header.bits,
            nonce: header.nonce,
            daa_score: header.daa_score,
            blue_work: header.blue_work.clone(),
            blue_score: header.blue_score,
            pruning_point: header.pruning_point.clone(),
        },
        transactions: block
            .transactions
            .iter()
            .map(|tx| {
                let verbose = tx.verbose_data.as_ref();
                Transaction {
                    transaction_id: verbose
                        .map(|v| v.transaction_id.clone())
                        .unwrap_or_default(),
                    hash: verbose.map(|v| v.hash.clone()).unwrap_or_default(),
                    mass: tx.mass,
                    inputs: tx
                        .inputs
                        .iter()
                        .map(|input| {
                            let outpoint = input.previous_outpoint.as_ref();
                            TransactionInputVerbose {
                                previous_outpoint: OutpointVerbose {
                                    transaction_id: outpoint
                                        .map(|o| o.transaction_id.clone())
                                        .unwrap_or_default(),
                                    index: outpoint.map(|o| o.index).unwrap_or(0),
                                },
                                signature_script: input.signature_script.clone(),
                                sequence: input.sequence,
                            }
                        })
                        .collect(),
                    outputs: tx
                        .outputs
                        .iter()
                        .map(|output| {
                            let script_pk = output.script_public_key.as_ref();
                            TransactionOutput {
                                amount: output.amount,
                                script_public_key: script_pk
                                    .map(|s| s.script_public_key.clone())
                                    .unwrap_or_default(),
                            }
                        })
                        .collect(),
                }
            })
            .collect(),
        verbose_data: block.verbose_data.as_ref().map(|vd| BlockVerboseData {
            hash: vd.hash.clone(),
            difficulty: vd.difficulty,
            selected_parent_hash: vd.selected_parent_hash.clone(),
            transaction_ids: vd.transaction_ids.clone(),
            is_header_only: vd.is_header_only,
            blue_score: vd.blue_score,
            is_chain_block: vd.is_chain_block,
        }),
    };

    Ok(Json(RpcResponse::success(block_response, latency_ms)))
}

/// Submit transaction to the network
pub async fn submit_transaction(
    State(state): State<AppState>,
    Json(request): Json<SubmitTransactionRequest>,
) -> Result<Json<RpcResponse<SubmitTransactionResponse>>, RpcError> {
    let start = Instant::now();

    // Convert JSON transaction to proto format
    let proto_tx = convert_to_proto_transaction(request.transaction)?;

    // Submit to Kaspa node
    let response = state
        .kaspa_client
        .submit_transaction(proto_tx, request.allow_orphan)
        .await?;

    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
    metrics::record_latency("submit_transaction", latency_ms);

    let submit_response = SubmitTransactionResponse {
        transaction_id: response.transaction_id,
    };

    Ok(Json(RpcResponse::success(submit_response, latency_ms)))
}

/// Get DAG tips (virtual selected parent chain)
pub async fn get_dag_tips(
    State(state): State<AppState>,
) -> Result<Json<RpcResponse<DAGTipsResponse>>, RpcError> {
    let start = Instant::now();

    let response = state.kaspa_client.get_dag_tips().await?;

    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
    metrics::record_latency("get_dag_tips", latency_ms);

    let dag_response = DAGTipsResponse {
        tip_hashes: response.tip_hashes,
        block_count: response.block_count,
        header_count: response.header_count,
        difficulty: response.difficulty,
        past_median_time: response.past_median_time,
        virtual_parent_hashes: response.virtual_parent_hashes,
        pruning_point_hash: response.pruning_point_hash,
        virtual_daa_score: response.virtual_daa_score,
    };

    Ok(Json(RpcResponse::success(dag_response, latency_ms)))
}

/// Helper: Validate hash format (64 hex chars)
fn is_valid_hash(hash: &str) -> bool {
    hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())
}

/// Helper: Convert JSON transaction to proto format
fn convert_to_proto_transaction(
    tx: TransactionInput,
) -> Result<client::proto::RpcTransaction, RpcError> {
    use client::proto::*;

    Ok(RpcTransaction {
        version: tx.version.unwrap_or(0),
        inputs: tx
            .inputs
            .into_iter()
            .map(|input| RpcTransactionInput {
                previous_outpoint: Some(RpcOutpoint {
                    transaction_id: input.previous_outpoint.transaction_id,
                    index: input.previous_outpoint.index,
                }),
                signature_script: input.signature_script,
                sequence: input.sequence,
                sig_op_count: input.sig_op_count.unwrap_or(0),
                verbose_data: None,
            })
            .collect(),
        outputs: tx
            .outputs
            .into_iter()
            .map(|output| RpcTransactionOutput {
                amount: output.amount,
                script_public_key: Some(RpcScriptPublicKey {
                    script_public_key: output.script_public_key.script_public_key,
                    version: output.script_public_key.version as u32,
                }),
                verbose_data: None,
            })
            .collect(),
        lock_time: tx.lock_time.unwrap_or(0),
        subnetwork_id: tx.subnetwork_id.unwrap_or_default(),
        gas: tx.gas.unwrap_or(0),
        payload: tx.payload.unwrap_or_default(),
        mass: 0, // Will be calculated by node
        verbose_data: None,
    })
}
