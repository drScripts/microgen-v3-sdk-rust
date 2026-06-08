use crate::auth::check_status;
use crate::error::Result;
use crate::types::Storage;
use std::sync::{Arc, Mutex};

/// File storage client.
pub struct StorageClient {
    client: reqwest::Client,
    upload_url: String,
    token: Arc<Mutex<Option<String>>>,
}

impl StorageClient {
    pub(crate) fn new(
        client: reqwest::Client,
        upload_url: String,
        token: Arc<Mutex<Option<String>>>,
    ) -> Self {
        Self {
            client,
            upload_url,
            token,
        }
    }

    /// Upload a file.
    ///
    /// - `data` – raw file bytes.
    /// - `file_name` – the file name (required when `data` is a raw blob).
    /// - `token` – optional bearer token override (takes precedence over stored token).
    pub async fn upload(
        &self,
        data: Vec<u8>,
        file_name: &str,
        token: Option<&str>,
    ) -> Result<Storage> {
        let url = format!("{}/upload", self.upload_url);

        let part = reqwest::multipart::Part::bytes(data)
            .file_name(file_name.to_string());
        let form = reqwest::multipart::Form::new().part("file", part);

        let mut req = self.client.post(&url).multipart(form);

        if let Some(t) = token {
            req = req.header(reqwest::header::AUTHORIZATION, format!("Bearer {}", t));
        } else if let Some(t) = self.token.lock().ok().and_then(|g| g.clone()) {
            req = req.header(reqwest::header::AUTHORIZATION, format!("Bearer {}", t));
        }

        let resp = req.send().await?;
        let resp = check_status(resp).await?;
        Ok(resp.json().await?)
    }
}
