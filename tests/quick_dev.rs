use anyhow::Result;
use reqwest::{multipart, Client};
use serde_json::{json, Value};
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use log::{error, info};
use env_logger;

#[tokio::test]
async fn quick_dev() -> Result<()> {
    // Initialize the logger
    let _ = env_logger::builder().is_test(true).try_init();

    //----------------------------------
    // 1) Setup: Build an admin client
    //----------------------------------
    let admin_client = Client::builder()
        .cookie_store(true)
        .build()?;

    // Helper function to print response details
    async fn print_response(res: reqwest::Response, description: &str) -> Result<Value> {
        let status = res.status();
        let headers = res.headers().clone();
        let cookies = res
            .cookies()
            .map(|c| format!("{}={}", c.name(), c.value()))
            .collect::<Vec<String>>()
            .join("; ");

        let body_text = res.text().await.unwrap_or_else(|_| "<Failed to read body>".to_string());

        println!("=== Response for {description} ===");
        println!("=> Status         : {}", status);
        println!("=> Headers        :");
        for (key, value) in headers.iter() {
            println!("   {}: {}", key, value.to_str().unwrap_or("<invalid utf8>"));
        }
        if !cookies.is_empty() {
            println!("=> Client Cookies : {}", cookies);
        }
        println!("=> Response Body  :\n{}\n===\n", body_text);

        let body_json = serde_json::from_str(&body_text).unwrap_or(Value::Null);
        Ok(body_json)
    }

    //----------------------------------
    // 2) Basic "hello" endpoints
    //----------------------------------
    info!("Sending GET /hello?name=Diana");
    let res = admin_client
        .get("http://localhost:8000/hello")
        .query(&[("name", "Diana")])
        .send()
        .await?;
    print_response(res, "GET /hello?name=Diana").await?;

    info!("Sending GET /hello2/Rafa");
    let res = admin_client
        .get("http://localhost:8000/hello2/Rafa")
        .send()
        .await?;
    print_response(res, "GET /hello2/Rafa").await?;

    //----------------------------------
    // 3) Admin login
    //----------------------------------
    info!("Sending POST /api/login (admin)");
    let res = admin_client
        .post("http://localhost:8000/api/login")
        .json(&json!({
            "username": "admin",
            "password": "welcome"
        }))
        .send()
        .await?;
    let login_response: Value = print_response(res, "POST /api/login (admin)").await?;

    if !login_response["result"]["success"].as_bool().unwrap_or(false) {
        error!("Admin login failed");
        return Err(anyhow::anyhow!("Admin login failed"));
    }
    let admin_id = login_response["result"]["user_id"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("Admin ID not found"))?;
    info!("Logged in as admin with ID: {}", admin_id);

    //----------------------------------
    // 4) Create multiple users
    //----------------------------------
    info!("Sending POST /api/users (User 1)");
    let res = admin_client
        .post("http://localhost:8000/api/users")
        .json(&json!({
            "username": "User 1",
            "pwd_clear": "welcome1"
        }))
        .send()
        .await?;
    let user1: Value = print_response(res, "POST /api/users (User 1)").await?;
    let user1_id = user1["id"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("User 1 ID not found"))?;
    info!("Captured User 1 ID: {}", user1_id);

    info!("Sending POST /api/users (User 2)");
    let res = admin_client
        .post("http://localhost:8000/api/users")
        .json(&json!({
            "username": "User 2",
            "pwd_clear": "welcome2"
        }))
        .send()
        .await?;
    let user2: Value = print_response(res, "POST /api/users (User 2)").await?;
    let user2_id = user2["id"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("User 2 ID not found"))?;
    info!("Captured User 2 ID: {}", user2_id);

    // List all users
    info!("Sending GET /api/users");
    let res = admin_client
        .get("http://localhost:8000/api/users")
        .send()
        .await?;
    print_response(res, "GET /api/users").await?;

    // Retrieve user 1
    info!("Sending GET /api/users/{}", user1_id);
    let res = admin_client
        .get(&format!("http://localhost:8000/api/users/{}", user1_id))
        .send()
        .await?;
    print_response(res, &format!("GET /api/users/{}", user1_id)).await?;

    // Update user1's username
    info!("Sending PUT /api/users/{}", user1_id);
    let res = admin_client
        .put(&format!("http://localhost:8000/api/users/{}", user1_id))
        .json(&json!({
            "username": "User 1 - Updated"
        }))
        .send()
        .await?;
    print_response(res, &format!("PUT /api/users/{}", user1_id)).await?;

    // List users again
    info!("Sending GET /api/users");
    let res = admin_client
        .get("http://localhost:8000/api/users")
        .send()
        .await?;
    print_response(res, "GET /api/users").await?;

    // Delete user2
    info!("Sending DELETE /api/users/{}", user2_id);
    let res = admin_client
        .delete(&format!("http://localhost:8000/api/users/{}", user2_id))
        .send()
        .await?;
    print_response(res, &format!("DELETE /api/users/{}", user2_id)).await?;

    // List users again
    info!("Sending GET /api/users");
    let res = admin_client
        .get("http://localhost:8000/api/users")
        .send()
        .await?;
    print_response(res, "GET /api/users").await?;

    //----------------------------------
    // 5) Tasks CRUD
    //----------------------------------
    info!("Sending POST /api/tasks (Task 1)");
    let res = admin_client
        .post("http://localhost:8000/api/tasks")
        .json(&json!({ "title": "Task 1" }))
        .send()
        .await?;
    let task1: Value = print_response(res, "POST /api/tasks (Task 1)").await?;
    let task1_id = task1["id"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("Task 1 ID not found"))?;
    info!("Captured Task 1 ID: {}", task1_id);

    info!("Sending POST /api/tasks (Task 2)");
    let res = admin_client
        .post("http://localhost:8000/api/tasks")
        .json(&json!({ "title": "Task 2" }))
        .send()
        .await?;
    let task2: Value = print_response(res, "POST /api/tasks (Task 2)").await?;
    let task2_id = task2["id"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("Task 2 ID not found"))?;
    info!("Captured Task 2 ID: {}", task2_id);

    // List tasks
    info!("Sending GET /api/tasks");
    let res = admin_client
        .get("http://localhost:8000/api/tasks")
        .send()
        .await?;
    print_response(res, "GET /api/tasks").await?;

    // Get Task 1
    info!("Sending GET /api/tasks/{}", task1_id);
    let res = admin_client
        .get(&format!("http://localhost:8000/api/tasks/{}", task1_id))
        .send()
        .await?;
    print_response(res, &format!("GET /api/tasks/{}", task1_id)).await?;

    // Update Task 1
    info!("Sending PUT /api/tasks/{}", task1_id);
    let res = admin_client
        .put(&format!("http://localhost:8000/api/tasks/{}", task1_id))
        .json(&json!({ "title": "Task 1 - Updated" }))
        .send()
        .await?;
    print_response(res, &format!("PUT /api/tasks/{}", task1_id)).await?;

    // List tasks again
    info!("Sending GET /api/tasks");
    let res = admin_client
        .get("http://localhost:8000/api/tasks")
        .send()
        .await?;
    print_response(res, "GET /api/tasks").await?;

    // Delete Task 2
    info!("Sending DELETE /api/tasks/{}", task2_id);
    let res = admin_client
        .delete(&format!("http://localhost:8000/api/tasks/{}", task2_id))
        .send()
        .await?;
    print_response(res, &format!("DELETE /api/tasks/{}", task2_id)).await?;

    // List tasks again
    info!("Sending GET /api/tasks");
    let res = admin_client
        .get("http://localhost:8000/api/tasks")
        .send()
        .await?;
    print_response(res, "GET /api/tasks").await?;

    //----------------------------------
    // 6) Document Upload Tests (Single-file)
    //----------------------------------
    info!("Preparing to upload documents of various types");
    let test_files_dir = "./tests/test_files";
    if !Path::new(test_files_dir).exists() {
        tokio::fs::create_dir_all(test_files_dir).await?;
        info!("Created test_files directory at {}", test_files_dir);
    }

    // A small variety of test documents
    let test_files: Vec<(&str, &str, &[u8])> = vec![
        ("test_document.txt", "text/plain", b"This is a test text document."),
        (
            "test_document.pdf",
            "application/pdf",
            // Make sure `tests/test_files/test_document.pdf` exists or is embedded
            include_bytes!("test_files/test_document.pdf") as &[u8],
        ),
    ];

    for (filename, mime_type, content) in test_files.iter() {
        let test_file_path = format!("{}/{}", test_files_dir, filename);
        if !Path::new(&test_file_path).exists() {
            tokio::fs::write(&test_file_path, content).await?;
            info!("Created test file at {}", test_file_path);
        }

        // Read the file into memory
        let mut file = File::open(&test_file_path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        // Create a multipart form (single file per request)
        let form = multipart::Form::new()
            .part(
                "file",
                multipart::Part::bytes(buffer.clone())
                    .file_name(filename.to_string())
                    .mime_str(mime_type)?,
            )
            .text("description", format!("This is a test upload for {}", filename));

        // Attempt upload (this hits your multi-file route, but with only one file)
        info!("Uploading document to /api/documents/upload (single-file): {}", filename);
        let res = admin_client
            .post("http://localhost:8000/api/documents/upload")
            .multipart(form)
            .send()
            .await?;

        if res.status().is_success() {
            let upload_response: Value =
                print_response(res, &format!("POST /api/documents/upload ({})", filename)).await?;

            // Since your route for multiple files returns an array, we expect an array in the response
            // Make sure to handle that
            let empty_vec = Vec::new();
            let docs_array = upload_response.as_array().unwrap_or(&empty_vec);
            if docs_array.is_empty() {
                error!("Upload returned an empty array for a single file.");
                continue;
            }

            // We only uploaded 1 file, so let's take the first doc
            let first_doc = &docs_array[0];
            let document_id = first_doc["id"]
                .as_i64()
                .ok_or_else(|| anyhow::anyhow!("Document ID not found"))?;
            info!("Captured Document ID: {}", document_id);

            // Retrieve
            info!("Retrieving /api/documents/{}", document_id);
            let res = admin_client
                .get(&format!("http://localhost:8000/api/documents/{}", document_id))
                .send()
                .await?;
            print_response(res, &format!("GET /api/documents/{}", document_id)).await?;

            // Update
            info!("Updating document metadata via PUT /api/documents/{}", document_id);
            let res = admin_client
                .put(&format!("http://localhost:8000/api/documents/{}", document_id))
                .json(&json!({
                    "filename": "updated_test_document.txt",
                    "filepath": "/new/path/updated_test_document.txt"
                }))
                .send()
                .await?;
            print_response(res, &format!("PUT /api/documents/{}", document_id)).await?;

            // Delete
            info!("Deleting document via DELETE /api/documents/{}", document_id);
            let res = admin_client
                .delete(&format!("http://localhost:8000/api/documents/{}", document_id))
                .send()
                .await?;
            print_response(res, &format!("DELETE /api/documents/{}", document_id)).await?;
        } else {
            let status = res.status();
            let error_response: Value = res.json().await.unwrap_or_else(|_| Value::Null);
            println!("=== Failed to Upload Document ===");
            println!("Status: {}", status);
            println!("Response: {:#?}", error_response);
            println!("===\n");
        }
    }

    //----------------------------------
    // 6b) Document Upload Tests (Multi-file in one request)
    //----------------------------------
    info!("Testing multiple-file upload in a single request");

    let multi_txt_path = format!("{}/multi1.txt", test_files_dir);
    let multi_pdf_path = format!("{}/multi2.pdf", test_files_dir);

    // Write the multi test files if they don't exist
    if !Path::new(&multi_txt_path).exists() {
        tokio::fs::write(&multi_txt_path, b"Multi-file text content").await?;
    }
    if !Path::new(&multi_pdf_path).exists() {
        tokio::fs::write(&multi_pdf_path, include_bytes!("test_files/test_document.pdf")).await?;
    }

    // Load them into memory
    let mut file_txt = File::open(&multi_txt_path).await?;
    let mut buffer_txt = Vec::new();
    file_txt.read_to_end(&mut buffer_txt).await?;

    let mut file_pdf = File::open(&multi_pdf_path).await?;
    let mut buffer_pdf = Vec::new();
    file_pdf.read_to_end(&mut buffer_pdf).await?;

    // Build a single multipart form with TWO files
    let multiple_form = multipart::Form::new()
        .part(
            "file",
            multipart::Part::bytes(buffer_txt)
                .file_name("multi1.txt")
                .mime_str("text/plain")?,
        )
        .part(
            "file",
            multipart::Part::bytes(buffer_pdf)
                .file_name("multi2.pdf")
                .mime_str("application/pdf")?,
        );

    let res = admin_client
        .post("http://localhost:8000/api/documents/upload")
        .multipart(multiple_form)
        .send()
        .await?;
    let multi_upload_resp: Value =
        print_response(res, "POST /api/documents/upload (multiple files)").await?;

    let empty_vec = Vec::new();
    let docs_array = multi_upload_resp.as_array().unwrap_or(&empty_vec);
    if docs_array.is_empty() {
        error!("Multiple-file upload returned an empty array.");
    } else {
        info!("Multiple-file upload returned {} documents", docs_array.len());
        for (idx, doc_val) in docs_array.iter().enumerate() {
            let doc_id = doc_val["id"].as_i64().unwrap_or(-1);
            if doc_id < 0 {
                error!("No valid doc_id returned for item index {}", idx);
                continue;
            }
            // Just do a simple check or retrieval
            info!("Retrieving /api/documents/{}", doc_id);
            let res = admin_client
                .get(&format!("http://localhost:8000/api/documents/{}", doc_id))
                .send()
                .await?;
            print_response(res, &format!("GET /api/documents/{}", doc_id)).await?;

            // Demonstrate delete to clean up
            info!("Deleting doc_id={}", doc_id);
            let res = admin_client
                .delete(&format!("http://localhost:8000/api/documents/{}", doc_id))
                .send()
                .await?;
            print_response(res, &format!("DELETE /api/documents/{}", doc_id)).await?;
        }
    }

    // Confirm no leftover documents
    info!("Listing documents via GET /api/documents");
    let res = admin_client
        .get("http://localhost:8000/api/documents")
        .send()
        .await?;
    print_response(res, "GET /api/documents").await?;

    info!("Sending POST /api/query/data");
    let res = admin_client
        .post("http://localhost:8000/api/query/data")
        .json(&json!({
            "prompt": "Any info about finances?"
        }))
        .send()
        .await?;
    let query_data_response: Value = print_response(res, "POST /api/query/data").await?;
    info!("Query data response: {:#?}", query_data_response);

    //----------------------------------
    // 7) Test "fine-tune"
    //----------------------------------
    info!("Sending POST /api/fine-tune");
    let res = admin_client
        .post("http://localhost:8000/api/fine-tune")
        .json(&json!({
            "prompt": "How to handle finances?",
            "context": [
                "Document snippet #1 about finances",
                "Document snippet #2 about budgeting"
            ]
        }))
        .send()
        .await?;
    let fine_tune_response: Value = print_response(res, "POST /api/fine-tune").await?;
    info!("Fine-tune response: {:#?}", fine_tune_response);

    //--------------------------------------------
    // 8) Test user "me" endpoint & password update
    //--------------------------------------------

    // a) Build a fresh client for user1
    let user1_client = Client::builder()
        .cookie_store(true)
        .build()?;

    // b) Log in as user1 with the old password
    info!("Logging in as User 1 with old password");
    let res = user1_client
        .post("http://localhost:8000/api/login")
        .json(&json!({
            "username": "User 1 - Updated",
            "password": "welcome1"
        }))
        .send()
        .await?;
    let login_response: Value = print_response(res, "POST /api/login (User 1)").await?;
    if !login_response["result"]["success"].as_bool().unwrap_or(false) {
        error!("Login for User 1 failed");
        return Err(anyhow::anyhow!("Login for User 1 failed"));
    }

    // c) GET /api/users/me to confirm identity
    info!("Sending GET /api/users/me as User 1");
    let res = user1_client
        .get("http://localhost:8000/api/users/me")
        .send()
        .await?;
    let me_response = print_response(res, "GET /api/users/me (User 1)").await?;
    info!("User 1's 'me' response: {:#?}", me_response);

    // d) Attempt to update password with old_password -> new_password
    info!("Sending PUT /api/users/{}/password to change password", user1_id);
    let res = user1_client
        .put(&format!("http://localhost:8000/api/users/{}/password", user1_id))
        .json(&json!({
            "old_password": "welcome1",
            "new_password": "updated123"
        }))
        .send()
        .await?;
    print_response(
        res,
        &format!("PUT /api/users/{}/password (User 1)", user1_id),
    ).await?;

    // e) Log in again with the new password
    info!("Re-logging in as User 1 with new password");
    let user1_client_new = Client::builder()
        .cookie_store(true)
        .build()?;
    let res = user1_client_new
        .post("http://localhost:8000/api/login")
        .json(&json!({
            "username": "User 1 - Updated",
            "password": "updated123"
        }))
        .send()
        .await?;
    let relogin_response: Value = print_response(res, "POST /api/login (User 1, new pwd)").await?;
    if !relogin_response["result"]["success"].as_bool().unwrap_or(false) {
        error!("Re-login for User 1 with new password failed");
        return Err(anyhow::anyhow!("Re-login with new password failed"));
    }

    // f) Confirm the 'me' route works with the new password
    info!("GET /api/users/me after password update");
    let res = user1_client_new
        .get("http://localhost:8000/api/users/me")
        .send()
        .await?;
    let me_response_after = print_response(res, "GET /api/users/me (User 1, new pwd)").await?;
    info!("User 1's 'me' response after password update: {:#?}", me_response_after);

    // g) Access protected route (admin/metrics) as User 1
    info!("Accessing /admin/metrics as user1 (admin role) ...");
    let res = user1_client
        .get("http://localhost:8000/admin/metrics")
        .send()
        .await?;
    print_response(res, "GET /admin/metrics (User 1 NOT admin)").await?;

    // h) Access protected route (admin/statistics) as User 1
    info!("Accessing /admin/statistics as user1 (admin role) ...");
    let res = user1_client
        .get("http://localhost:8000/admin/statistics")
        .send()
        .await?;
    print_response(res, "GET /admin/statistics (User 1 NOT admin)").await?;

    // i) Log off User 1
    info!("Logging off admin user");
    let res = admin_client
        .post("http://localhost:8000/api/logoff")
        .json(&json!({ "logoff": true }))
        .send()
        .await?;
    let logoff_response: Value = print_response(res, "POST /api/logoff (admin)").await?;

    if logoff_response["result"]["logged_off"].as_bool().unwrap_or(false) {
        info!("Admin successfully logged off.");
    } else {
        error!("Admin logoff failed.");
        return Err(anyhow::anyhow!("Admin logoff failed"));
    }

    // j) Attempt to access a protected route after logoff to ensure cookies are cleared
    info!("Attempting to access protected route /api/users after admin logoff");
    let res = admin_client
        .get("http://localhost:8000/api/users")
        .send()
        .await?;
    let protected_response: Value = print_response(res, "GET /api/users after admin logoff").await?;
    // Depending on your API's behavior, you might expect an unauthorized error
    if protected_response["error"].is_null() {
        error!("Protected route accessed even after logoff.");
        return Err(anyhow::anyhow!("Protected route should not be accessible after logoff."));
    } else {
        info!("Protected route access correctly denied after logoff.");
    }

    // c) Logoff user1_client_new
    info!("Logging off User 1");
    let res = user1_client_new
        .post("http://localhost:8000/api/logoff")
        .json(&json!({ "logoff": true }))
        .send()
        .await?;
    let logoff_response_user1: Value = print_response(res, "POST /api/logoff (User 1)").await?;

    if logoff_response_user1["result"]["logged_off"].as_bool().unwrap_or(false) {
        info!("User 1 successfully logged off.");
    } else {
        error!("User 1 logoff failed.");
        return Err(anyhow::anyhow!("User 1 logoff failed"));
    }

    // d) Attempt to access the 'me' endpoint after logoff to ensure cookies are cleared
    info!("Attempting to access /api/users/me after User 1 logoff");
    let res = user1_client_new
        .get("http://localhost:8000/api/users/me")
        .send()
        .await?;
    let me_after_logoff = print_response(res, "GET /api/users/me after User 1 logoff").await?;
    // Depending on your API's behavior, you might expect an unauthorized error or similar
    if me_after_logoff["error"].is_null() {
        error!("'me' endpoint accessed even after logoff.");
        return Err(anyhow::anyhow!("'me' endpoint should not be accessible after logoff."));
    } else {
        info!("'me' endpoint access correctly denied after logoff.");
    }

    info!("Sending POST /api/login (admin)");
    let res = admin_client
        .post("http://localhost:8000/api/login")
        .json(&json!({
            "username": "admin",
            "password": "welcome"
        }))
        .send()
        .await?;
    let login_response: Value = print_response(res, "POST /api/login (admin)").await?;

    if !login_response["result"]["success"].as_bool().unwrap_or(false) {
        error!("Admin login failed");
        return Err(anyhow::anyhow!("Admin login failed"));
    }
    let admin_id = login_response["result"]["user_id"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("Admin ID not found"))?;
    info!("Logged in as admin with ID: {}", admin_id);

    info!("Sending POST /api/users (User 3 with role=admin)");
    let res = admin_client
        .post("http://localhost:8000/api/users")
        .json(&json!({
        "username": "User 3",
        "pwd_clear": "welcome3",
        "role": "admin"
    }))
        .send()
        .await?;
    let user3: Value = print_response(res, "POST /api/users (User 3, admin)").await?;
    let user3_id = user3["id"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("User 3 ID not found"))?;
    info!("Captured User 3 ID: {}", user3_id);

    // Now log off the default admin
    info!("Logging off the original admin user");
    let res = admin_client
        .post("http://localhost:8000/api/logoff")
        .json(&json!({ "logoff": true }))
        .send()
        .await?;
    print_response(res, "POST /api/logoff (default admin)")
        .await?;

    // Build a new client for user3
    let user3_client = Client::builder()
        .cookie_store(true)
        .build()?;

    // Login as user3
    info!("Logging in as User 3 (who is also admin) with password welcome3");
    let res = user3_client
        .post("http://localhost:8000/api/login")
        .json(&json!({
        "username": "User 3",
        "password": "welcome3"
    }))
        .send()
        .await?;
    let user3_login: Value = print_response(res, "POST /api/login (User 3, admin role)").await?;
    if !user3_login["result"]["success"].as_bool().unwrap_or(false) {
        error!("User 3 login failed");
        return Err(anyhow::anyhow!("User 3 login failed"));
    }
    info!("User 3 logged in successfully.");

    // Check that user3 can access /admin endpoints:
    info!("Accessing /admin/metrics as user3 (admin role) ...");
    let res = user3_client
        .get("http://localhost:8000/admin/metrics")
        .send()
        .await?;
    print_response(res, "GET /admin/metrics (User 3 admin)").await?;

    info!("Accessing /admin/statistics as user3 (admin role) ...");
    let res = user3_client
        .get("http://localhost:8000/admin/statistics")
        .send()
        .await?;
    print_response(res, "GET /admin/statistics (User 3 admin)").await?;

    info!("=== ALL QUICK_DEV TESTS COMPLETED SUCCESSFULLY ===");
    Ok(())
}
