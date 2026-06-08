use microgen_v3_sdk_rust::{MicrogenClient, MicrogenClientOptions};

// ──────────────────────────────────────────────
//  Integration tests – require API_KEY env var
//  Run with: API_KEY=xxx cargo test -- --ignored
// ──────────────────────────────────────────────

/// Helper to get the API key from environment.
fn get_api_key() -> String {
    std::env::var("API_KEY").expect("API_KEY env var required for integration tests")
}

#[test]
#[ignore]
fn test_client_creation() {
    let mg = MicrogenClient::new(MicrogenClientOptions::new("test-key"));
    // Verify the client was created without panicking
    assert!(mg.auth.token().is_none());
}

#[tokio::test]
#[ignore]
async fn test_auth_flow() {
    let api_key = get_api_key();
    let mg = MicrogenClient::new(MicrogenClientOptions::new(&api_key));

    // The actual API endpoint would need a registered user,
    // but this validates the request plumbing works.
    let result = mg
        .auth
        .login::<serde_json::Value>(&serde_json::json!({
            "email": "test@example.com",
            "password": "wrong-password",
        }))
        .await;

    // Should fail with auth error (not a network/connection error)
    match result {
        Ok(_) => panic!("expected error for wrong credentials"),
        Err(e) => {
            let err_str = e.to_string();
            // Should be an API error (4xx), not a connection error
            assert!(
                err_str.contains("API error"),
                "expected API error, got: {}",
                err_str
            );
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_database_crud() {
    let api_key = get_api_key();
    let mg = MicrogenClient::new(MicrogenClientOptions::new(&api_key));

    let posts = mg.service("posts");

    // Create
    let created = posts
        .create::<serde_json::Value>(
            &serde_json::json!({ "title": "Test Post", "content": "Hello" }),
            None,
        )
        .await;
    match created {
        Ok(resp) => {
            let id = resp.data.as_ref().and_then(|d| d["_id"].as_str())
                .expect("created record should have _id")
                .to_string();

            // Find
            let found = posts
                .find::<serde_json::Value>(None, None)
                .await
                .expect("find should succeed");
            assert!(found.data.is_some());

            // Get by ID
            let by_id = posts
                .get_by_id::<serde_json::Value>(&id, None, None)
                .await
                .expect("getById should succeed");
            assert!(by_id.data.is_some());

            // Update
            let updated = posts
                .update_by_id::<serde_json::Value>(
                    &id,
                    &serde_json::json!({ "title": "Updated" }),
                    None,
                )
                .await
                .expect("updateById should succeed");
            assert!(updated.data.is_some());

            // Delete
            let deleted = posts
                .delete_by_id::<serde_json::Value>(&id, None)
                .await
                .expect("deleteById should succeed");
            assert!(deleted.data.is_some());
        }
        Err(e) => {
            // If it's a 404, the "posts" table doesn't exist – that's OK for this test
            if e.to_string().contains("404") {
                eprintln!("Table 'posts' not found (expected if not configured)");
            } else {
                panic!("Unexpected error: {}", e);
            }
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_auth_register() {
    let api_key = get_api_key();
    let mg = MicrogenClient::new(MicrogenClientOptions::new(&api_key));

    let result = mg
        .auth
        .register::<serde_json::Value>(&serde_json::json!({
            "firstName": "Test",
            "lastName": "User",
            "email": "rust-test@example.com",
            "password": "TestPass123!",
        }))
        .await;

    match result {
        Ok(resp) => {
            assert!(!resp.token.is_empty());
            assert!(mg.auth.token().is_some());
        }
        Err(e) => {
            // Registration may fail if user already exists or API config
            if e.to_string().contains("409") || e.to_string().contains("400") {
                eprintln!("Registration failed as expected (user may already exist): {}", e);
            } else {
                panic!("Unexpected error: {}", e);
            }
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_storage_upload() {
    let api_key = get_api_key();
    let mg = MicrogenClient::new(MicrogenClientOptions::new(&api_key));

    let content = b"Hello, Microgen from Rust!".to_vec();
    let result = mg.storage.upload(content, "hello.txt", None).await;

    match result {
        Ok(storage) => {
            assert!(!storage.url.is_empty());
            assert_eq!(storage.file_name, "hello.txt");
        }
        Err(e) => {
            if e.to_string().contains("401") || e.to_string().contains("403") {
                eprintln!("Auth required for storage: {}", e);
            } else {
                panic!("Unexpected error: {}", e);
            }
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_find_with_options() {
    let api_key = get_api_key();
    let mg = MicrogenClient::new(MicrogenClientOptions::new(&api_key));

    let posts = mg.service("posts");

    let result = posts
        .find::<serde_json::Value>(
            Some(&microgen_v3_sdk_rust::types::FindOption {
                limit: Some(5),
                skip: Some(0),
                ..Default::default()
            }),
            None,
        )
        .await;

    match result {
        Ok(resp) => {
            println!("Found {} posts", resp.data.as_ref().map_or(0, |d| d.len()));
        }
        Err(e) => {
            if e.to_string().contains("404") {
                eprintln!("Table 'posts' not found");
            } else {
                panic!("Unexpected error: {}", e);
            }
        }
    }
}
