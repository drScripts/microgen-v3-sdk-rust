use crate::auth::check_status;
use crate::error::Result;
use crate::field::FieldClient;
use crate::types::*;
use reqwest::Client;

/// CRUD client for a single table / service.
pub struct QueryClient {
    client: Client,
    table_url: String,
    headers: reqwest::header::HeaderMap,
    /// Field (schema) sub-client.
    pub field: FieldClient,
}

impl QueryClient {
    pub(crate) fn new(
        client: Client,
        table_name: &str,
        base_url: &str,
        headers: reqwest::header::HeaderMap,
    ) -> Self {
        let table_url = format!("{}/{}", base_url, table_name);
        let field = FieldClient::new(
            client.clone(),
            &format!("{}/tables/{}", base_url, table_name),
            &headers,
        );
        Self {
            client,
            table_url,
            headers,
            field,
        }
    }

    // ── query-string helpers ─────────────────────────────

    fn build_url(&self, query: &serde_json::Map<String, serde_json::Value>) -> String {
        if query.is_empty() {
            self.table_url.clone()
        } else {
            let qs = serde_qs::to_string(query).unwrap_or_default();
            format!("{}?{}", self.table_url, qs)
        }
    }

    fn build_url_id(
        &self,
        id: &str,
        query: &serde_json::Map<String, serde_json::Value>,
    ) -> String {
        let base = format!("{}/{}", self.table_url, id);
        if query.is_empty() {
            base
        } else {
            let qs = serde_qs::to_string(query).unwrap_or_default();
            format!("{}?{}", base, qs)
        }
    }

    fn auth_headers(&self, token: Option<&str>) -> reqwest::header::HeaderMap {
        let mut h = self.headers.clone();
        if let Some(t) = token {
            h.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", t).parse().unwrap(),
            );
        }
        h
    }

    fn bulk_header() -> reqwest::header::HeaderName {
        reqwest::header::HeaderName::from_static("x-bulk-response-type")
    }

    // ── public API ───────────────────────────────────────

    /// Find records with optional filters.
    pub async fn find<T: serde::de::DeserializeOwned>(
        &self,
        option: Option<&FindOption>,
        token: Option<&str>,
    ) -> Result<MicrogenResponse<T>> {
        let query = option.map(build_find_query).unwrap_or_default();
        let url = self.build_url(&query);
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers(token))
            .send()
            .await?;
        let resp = check_status(resp).await?;

        let limit = resp
            .headers()
            .get("x-pagination-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok());
        let skip = resp
            .headers()
            .get("x-pagination-skip")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok());

        let data: Vec<T> = resp.json().await?;
        Ok(MicrogenResponse {
            data: Some(data),
            limit,
            skip,
        })
    }

    /// Get a single record by ID.
    pub async fn get_by_id<T: serde::de::DeserializeOwned>(
        &self,
        id: &str,
        option: Option<&GetByIdOption>,
        token: Option<&str>,
    ) -> Result<MicrogenSingleResponse<T>> {
        let query = option.map(build_get_by_id_query).unwrap_or_default();
        let url = self.build_url_id(id, &query);
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers(token))
            .send()
            .await?;
        let resp = check_status(resp).await?;
        let data: T = resp.json().await?;
        Ok(MicrogenSingleResponse { data: Some(data) })
    }

    /// Create a single record.
    pub async fn create<T: serde::de::DeserializeOwned>(
        &self,
        body: &impl serde::Serialize,
        token: Option<&str>,
    ) -> Result<MicrogenSingleResponse<T>> {
        let resp = self
            .client
            .post(&self.table_url)
            .headers(self.auth_headers(token))
            .json(body)
            .send()
            .await?;
        let resp = check_status(resp).await?;
        let data: T = resp.json().await?;
        Ok(MicrogenSingleResponse { data: Some(data) })
    }

    /// Create multiple records at once.
    ///
    /// When `bulk_behavior` is `None` the full list of created records is returned.
    /// When set to `BulkBehavior::Count` only the count is returned.
    pub async fn create_many<T: serde::de::DeserializeOwned>(
        &self,
        body: &impl serde::Serialize,
        token: Option<&str>,
        bulk_behavior: Option<BulkBehavior>,
    ) -> Result<MicrogenResponse<T>> {
        let mut h = self.auth_headers(token);
        if let Some(b) = bulk_behavior {
            h.insert(Self::bulk_header(), b.as_header_value().parse().unwrap());
        }
        let resp = self
            .client
            .post(&self.table_url)
            .headers(h)
            .json(body)
            .send()
            .await?;
        let resp = check_status(resp).await?;
        let data: Vec<T> = resp.json().await?;
        Ok(MicrogenResponse {
            data: Some(data),
            limit: None,
            skip: None,
        })
    }

    /// Update a single record by ID. Supports `$inc` via [`UpdateBody`].
    pub async fn update_by_id<T: serde::de::DeserializeOwned>(
        &self,
        id: &str,
        body: &impl serde::Serialize,
        token: Option<&str>,
    ) -> Result<MicrogenSingleResponse<T>> {
        let url = format!("{}/{}", self.table_url, id);
        let resp = self
            .client
            .patch(&url)
            .headers(self.auth_headers(token))
            .json(body)
            .send()
            .await?;
        let resp = check_status(resp).await?;
        let data: T = resp.json().await?;
        Ok(MicrogenSingleResponse { data: Some(data) })
    }

    /// Update multiple records.
    ///
    /// When `bulk_behavior` is `None` the full list is returned.
    /// When set to `BulkBehavior::Count` only the count is returned.
    pub async fn update_many<T: serde::de::DeserializeOwned>(
        &self,
        body: &impl serde::Serialize,
        token: Option<&str>,
        bulk_behavior: Option<BulkBehavior>,
    ) -> Result<MicrogenResponse<T>> {
        let mut h = self.auth_headers(token);
        if let Some(b) = bulk_behavior {
            h.insert(Self::bulk_header(), b.as_header_value().parse().unwrap());
        }
        let resp = self
            .client
            .patch(&self.table_url)
            .headers(h)
            .json(body)
            .send()
            .await?;
        let resp = check_status(resp).await?;
        let data: Vec<T> = resp.json().await?;
        Ok(MicrogenResponse {
            data: Some(data),
            limit: None,
            skip: None,
        })
    }

    /// Delete a single record by ID.
    pub async fn delete_by_id<T: serde::de::DeserializeOwned>(
        &self,
        id: &str,
        token: Option<&str>,
    ) -> Result<MicrogenSingleResponse<T>> {
        let url = format!("{}/{}", self.table_url, id);
        let resp = self
            .client
            .delete(&url)
            .headers(self.auth_headers(token))
            .send()
            .await?;
        let resp = check_status(resp).await?;
        let data: T = resp.json().await?;
        Ok(MicrogenSingleResponse { data: Some(data) })
    }

    /// Delete multiple records by ID.
    ///
    /// When `bulk_behavior` is `None` the full list is returned.
    /// When set to `BulkBehavior::Count` only the count is returned.
    pub async fn delete_many<T: serde::de::DeserializeOwned>(
        &self,
        ids: &[String],
        token: Option<&str>,
        bulk_behavior: Option<BulkBehavior>,
    ) -> Result<MicrogenResponse<T>> {
        let mut h = self.auth_headers(token);
        if let Some(b) = bulk_behavior {
            h.insert(Self::bulk_header(), b.as_header_value().parse().unwrap());
        }
        let record_ids = ids.join(",");
        let url = format!("{}?recordIds={}", self.table_url, record_ids);
        let resp = self
            .client
            .delete(&url)
            .headers(h)
            .send()
            .await?;
        let resp = check_status(resp).await?;
        let data: Vec<T> = resp.json().await?;
        Ok(MicrogenResponse {
            data: Some(data),
            limit: None,
            skip: None,
        })
    }

    /// Link a related record.
    pub async fn link<T: serde::de::DeserializeOwned>(
        &self,
        id: &str,
        body: &impl serde::Serialize,
        token: Option<&str>,
    ) -> Result<MicrogenSingleResponse<T>> {
        let url = format!("{}/{}", self.table_url, id);
        let resp = self
            .client
            .request(reqwest::Method::from_bytes(b"LINK").unwrap(), &url)
            .headers(self.auth_headers(token))
            .json(body)
            .send()
            .await?;
        let resp = check_status(resp).await?;
        let data: T = resp.json().await?;
        Ok(MicrogenSingleResponse { data: Some(data) })
    }

    /// Unlink a related record.
    pub async fn unlink<T: serde::de::DeserializeOwned>(
        &self,
        id: &str,
        body: &impl serde::Serialize,
        token: Option<&str>,
    ) -> Result<MicrogenSingleResponse<T>> {
        let url = format!("{}/{}", self.table_url, id);
        let resp = self
            .client
            .request(reqwest::Method::from_bytes(b"UNLINK").unwrap(), &url)
            .headers(self.auth_headers(token))
            .json(body)
            .send()
            .await?;
        let resp = check_status(resp).await?;
        let data: T = resp.json().await?;
        Ok(MicrogenSingleResponse { data: Some(data) })
    }

    /// Count records, optionally filtered.
    pub async fn count(
        &self,
        option: Option<&CountOption>,
        token: Option<&str>,
    ) -> Result<MicrogenCountResponse> {
        let query = option.map(build_count_query).unwrap_or_default();
        let url = if query.is_empty() {
            format!("{}/count", self.table_url)
        } else {
            let qs = serde_qs::to_string(&query).unwrap_or_default();
            format!("{}/count?{}", self.table_url, qs)
        };
        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers(token))
            .send()
            .await?;
        let resp = check_status(resp).await?;
        let data: crate::types::MicrogenCount = resp.json().await?;
        Ok(MicrogenCountResponse {
            data: Some(data),
        })
    }
}
