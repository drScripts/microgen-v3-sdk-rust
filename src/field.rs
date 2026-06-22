use crate::auth::check_status;
use crate::error::Result;
use crate::types::*;
use reqwest::Client;
use serde::Deserialize;

/// Schema (field definition) client for a table.
#[derive(Clone)]
pub struct FieldClient {
    client: Client,
    fields_url: String,
    headers: reqwest::header::HeaderMap,
}

impl FieldClient {
    pub(crate) fn new(
        client: Client,
        base_url: &str,
        headers: &reqwest::header::HeaderMap,
    ) -> Self {
        Self {
            client,
            fields_url: format!("{}/fields", base_url),
            headers: headers.clone(),
        }
    }

    /// List all fields for the table.
    pub async fn find<T: serde::de::DeserializeOwned>(
        &self,
    ) -> Result<FieldResponse<T>> {
        let resp = self
            .client
            .get(&self.fields_url)
            .headers(self.headers.clone())
            .send()
            .await?;
        let resp = check_status(resp).await?;
        let data: Vec<Field<T>> = resp.json().await?;
        Ok(FieldResponse {
            data: Some(data),
        })
    }

    /// Get a single field by ID.
    pub async fn get_by_id<T: serde::de::DeserializeOwned>(
        &self,
        id: &str,
    ) -> Result<FieldSingleResponse<T>> {
        let url = format!("{}/{}", self.fields_url, id);
        let resp = self
            .client
            .get(&url)
            .headers(self.headers.clone())
            .send()
            .await?;
        let resp = check_status(resp).await?;
        let data: Field<T> = resp.json().await?;
        Ok(FieldSingleResponse {
            data: Some(data),
        })
    }

    /// Create a new field in the table schema.
    pub async fn create<T: serde::de::DeserializeOwned>(
        &self,
        body: &CreateFieldBody,
    ) -> Result<FieldSingleResponse<T>> {
        let resp = self
            .client
            .post(&self.fields_url)
            .headers(self.headers.clone())
            .json(body)
            .send()
            .await?;
        let resp = check_status(resp).await?;
        let data: Field<T> = resp.json().await?;
        Ok(FieldSingleResponse {
            data: Some(data),
        })
    }
}

/// Response for [`FieldClient::find()`].
#[derive(Debug, Clone, Deserialize)]
pub struct FieldResponse<T> {
    pub data: Option<Vec<Field<T>>>,
}

/// Response for [`FieldClient::get_by_id()`] / [`FieldClient::create()`].
#[derive(Debug, Clone, Deserialize)]
pub struct FieldSingleResponse<T> {
    pub data: Option<Field<T>>,
}
