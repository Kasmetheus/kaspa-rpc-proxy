use crate::error::RpcError;
use tonic::transport::Channel;

// Include generated protobuf code
pub mod proto {
    tonic::include_proto!("protowire");
}

use proto::{
    rpc_client::RpcClient, GetBlockRequestMessage, GetBlockDagInfoRequestMessage,
    GetUtxosByAddressesRequestMessage, KaspadRequest, KaspadResponse, 
    NotifyUtxosChangedRequestMessage, SubmitTransactionRequestMessage,
};

/// High-performance gRPC client for Kaspa node
pub struct KaspaClient {
    client: RpcClient<Channel>,
}

impl KaspaClient {
    /// Create new client connection to Kaspa node
    pub async fn new(endpoint: &str) -> Result<Self, RpcError> {
        let channel = Channel::from_shared(endpoint.to_string())
            .map_err(|e| RpcError::Connection(e.to_string()))?
            .connect()
            .await
            .map_err(|e| RpcError::Connection(e.to_string()))?;

        let client = RpcClient::new(channel);
        
        Ok(Self { client })
    }

    /// Get block by hash
    pub async fn get_block(
        &self,
        hash: String,
        include_transactions: bool,
    ) -> Result<proto::GetBlockResponseMessage, RpcError> {
        let request = KaspadRequest {
            id: generate_request_id(),
            payload: Some(proto::kaspad_request::Payload::GetBlockRequest(
                GetBlockRequestMessage {
                    hash,
                    include_transactions,
                },
            )),
        };

        let response = self
            .send_request(request)
            .await?;

        if let Some(proto::kaspad_response::Payload::GetBlockResponse(resp)) = response.payload {
            if let Some(error) = &resp.error {
                return Err(RpcError::Kaspa(error.message.clone()));
            }
            Ok(resp)
        } else {
            Err(RpcError::InvalidResponse("Expected GetBlockResponse".into()))
        }
    }

    /// Submit transaction to network
    pub async fn submit_transaction(
        &self,
        transaction: proto::RpcTransaction,
        allow_orphan: bool,
    ) -> Result<proto::SubmitTransactionResponseMessage, RpcError> {
        let request = KaspadRequest {
            id: generate_request_id(),
            payload: Some(proto::kaspad_request::Payload::SubmitTransactionRequest(
                SubmitTransactionRequestMessage {
                    transaction: Some(transaction),
                    allow_orphan,
                },
            )),
        };

        let response = self.send_request(request).await?;

        if let Some(proto::kaspad_response::Payload::SubmitTransactionResponse(resp)) =
            response.payload
        {
            if let Some(error) = &resp.error {
                return Err(RpcError::Kaspa(error.message.clone()));
            }
            Ok(resp)
        } else {
            Err(RpcError::InvalidResponse("Expected SubmitTransactionResponse".into()))
        }
    }

    /// Get DAG tips (chain heads)
    pub async fn get_dag_tips(&self) -> Result<proto::GetBlockDagInfoResponseMessage, RpcError> {
        let request = KaspadRequest {
            id: generate_request_id(),
            payload: Some(proto::kaspad_request::Payload::GetBlockDagInfoRequest(
                GetBlockDagInfoRequestMessage {},
            )),
        };

        let response = self.send_request(request).await?;

        if let Some(proto::kaspad_response::Payload::GetBlockDagInfoResponse(resp)) =
            response.payload
        {
            if let Some(error) = &resp.error {
                return Err(RpcError::Kaspa(error.message.clone()));
            }
            Ok(resp)
        } else {
            Err(RpcError::InvalidResponse("Expected GetBlockDagInfoResponse".into()))
        }
    }

    /// Get UTXOs by addresses
    pub async fn get_utxos_by_addresses(
        &self,
        addresses: Vec<String>,
    ) -> Result<proto::GetUtxosByAddressesResponseMessage, RpcError> {
        let request = KaspadRequest {
            id: generate_request_id(),
            payload: Some(proto::kaspad_request::Payload::GetUtxosByAddressesRequest(
                GetUtxosByAddressesRequestMessage { addresses },
            )),
        };

        let response = self.send_request(request).await?;

        if let Some(proto::kaspad_response::Payload::GetUtxosByAddressesResponse(resp)) =
            response.payload
        {
            if let Some(error) = &resp.error {
                return Err(RpcError::Kaspa(error.message.clone()));
            }
            Ok(resp)
        } else {
            Err(RpcError::InvalidResponse("Expected GetUtxosByAddressesResponse".into()))
        }
    }

    /// Subscribe to UTXO changes (for WebSocket streaming)
    pub async fn subscribe_utxo_changes(
        &self,
        addresses: Vec<String>,
    ) -> Result<tonic::Streaming<KaspadResponse>, RpcError> {
        let request = KaspadRequest {
            id: generate_request_id(),
            payload: Some(proto::kaspad_request::Payload::NotifyUtxosChangedRequest(
                NotifyUtxosChangedRequestMessage { 
                  command: 0,
                  addresses,
                },
            )),
        };

        let stream = self
            .client
            .clone()
            .message_stream(tokio_stream::once(request))
            .await
            .map_err(|e| RpcError::Connection(e.to_string()))?
            .into_inner();

        Ok(stream)
    }

    /// Internal helper to send request and get single response
    async fn send_request(&self, request: KaspadRequest) -> Result<KaspadResponse, RpcError> {
        use tokio_stream::StreamExt;

        let mut stream = self
            .client
            .clone()
            .message_stream(tokio_stream::once(request))
            .await
            .map_err(|e| RpcError::Connection(e.to_string()))?
            .into_inner();

        stream
            .next()
            .await
            .ok_or_else(|| RpcError::InvalidResponse("Empty response stream".into()))?
            .map_err(|e| RpcError::Connection(e.to_string()))
    }
}

/// Generate unique request ID
fn generate_request_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
