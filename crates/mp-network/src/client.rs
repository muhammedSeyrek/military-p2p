//! HTTP client for talking to other nodes.

use reqwest::Client as HttpClient;
use uuid::Uuid;

use mp_protocol::api::{DispatchOperationRequest, DispatchOperationResponse, FetchPartResponse};

use crate::error::{NetworkError, Result};

pub struct Client {
    http: HttpClient,
}

impl Client {
    pub fn new() -> Result<Self> {
        let http = HttpClient::builder()
            // Accept self-signed certs (TLS is disabled in the demo build anyway).
            .danger_accept_invalid_certs(true)
            .build()?;
        Ok(Self { http })
    }

    /// General → commander: `POST /api/operations`
    pub async fn dispatch_operation(
        &self,
        base_url: &str,
        req: &DispatchOperationRequest,
    ) -> Result<DispatchOperationResponse> {
        let url = format!("{}/api/operations", base_url.trim_end_matches('/'));
        let resp = self
            .http
            .post(&url)
            .json(req)
            .send()
            .await?
            .error_for_status()?
            .json::<DispatchOperationResponse>()
            .await?;
        Ok(resp)
    }

    /// Commander → commander: `GET /api/parts/{op_id}/{part_idx}`
    pub async fn fetch_part(
        &self,
        base_url: &str,
        operation_id: Uuid,
        part_index: usize,
    ) -> Result<FetchPartResponse> {
        let url = format!(
            "{}/api/parts/{}/{}",
            base_url.trim_end_matches('/'),
            operation_id,
            part_index
        );
        let resp = self.http.get(&url).send().await?;

        if resp.status() == 404 {
            return Err(NetworkError::NotFound);
        }

        let resp = resp.error_for_status()?.json::<FetchPartResponse>().await?;
        Ok(resp)
    }

    /// Health check — useful for debugging.
    pub async fn health_check(&self, base_url: &str) -> Result<()> {
        let url = format!("{}/health", base_url.trim_end_matches('/'));
        self.http.get(&url).send().await?.error_for_status()?;
        Ok(())
    }
}
