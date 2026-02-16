use reqwest::{Client, Method, header};
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};
use crate::orchid::Orchid;
use gloo_storage::{LocalStorage, Storage};
use web_sys::{FileReader, Blob};
use wasm_bindgen::JsCast;
use gloo_file::File;
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize, Debug)]
struct GithubFile {
    sha: String,
    // other fields omitted
}

#[derive(Serialize, Debug)]
struct UpdateFilePayload {
    message: String,
    content: String,
    sha: Option<String>,
    branch: String,
}

#[derive(Serialize)]
struct LfsBatchRequest {
    operation: String,
    transfers: Vec<String>,
    objects: Vec<LfsObject>,
}

#[derive(Serialize)]
struct LfsObject {
    oid: String,
    size: i64,
}

#[derive(Deserialize)]
struct LfsBatchResponse {
    objects: Vec<LfsObjectResponse>,
}

#[derive(Deserialize)]
struct LfsObjectResponse {
    actions: Option<LfsActions>,
}

#[derive(Deserialize)]
struct LfsActions {
    upload: Option<LfsAction>,
}

#[derive(Deserialize)]
struct LfsAction {
    href: String,
    header: Option<std::collections::HashMap<String, String>>,
}

const API_BASE: &str = "https://api.github.com";

fn get_config() -> Option<(String, String, String)> {
    let token: String = LocalStorage::get("github_token").ok()?;
    let owner: String = LocalStorage::get("repo_owner").ok()?;
    let repo: String = LocalStorage::get("repo_name").ok()?;
    if token.is_empty() || owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((token, owner, repo))
}

pub async fn upload_image_to_github(file_name: String, file_data: Vec<u8>) -> Result<String, String> {
    let (token, owner, repo) = get_config().ok_or("GitHub configuration missing")?;
    let client = Client::new();

    // 1. Calculate SHA256 and Size for LFS
    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let result = hasher.finalize();
    let oid = hex::encode(result);
    let size = file_data.len() as i64;

    // 2. Request LFS Batch Upload
    let lfs_url = format!("https://github.com/{}/{}.git/info/lfs/objects/batch", owner, repo);
    let batch_req = LfsBatchRequest {
        operation: "upload".to_string(),
        transfers: vec!["basic".to_string()],
        objects: vec![LfsObject { oid: oid.clone(), size }],
    };

    let batch_resp = client.post(&lfs_url)
        .header("Authorization", format!("Bearer {}", token)) // Basic auth or Bearer work? Usually Basic for git ops but Bearer for API. Let's try Basic with empty user? No, API token as Basic user.
        // Actually, GitHub LFS batch API works with Authorization: Basic base64(username:token) OR just the token if it's OAuth/PAT?
        // Let's try standard Bearer first, if not, Basic.
        // Wait, standard GitHub API uses Bearer/Token. LFS endpoint is technically part of the git server.
        // Usually it expects Basic Auth with the token as password.
        .basic_auth("git", Some(&token)) // Username 'git' often works or empty.
        .header("Accept", "application/vnd.git-lfs+json")
        .header("Content-Type", "application/vnd.git-lfs+json")
        .json(&batch_req)
        .send()
        .await
        .map_err(|e| format!("LFS Batch Init Failed: {}", e))?;

    if !batch_resp.status().is_success() {
        return Err(format!("LFS Init Error: {}", batch_resp.status()));
    }

    let batch_data: LfsBatchResponse = batch_resp.json().await.map_err(|e| e.to_string())?;
    
    // 3. Upload Blob if needed
    if let Some(obj) = batch_data.objects.first() {
        if let Some(actions) = &obj.actions {
            if let Some(upload) = &actions.upload {
                let mut put_req = client.put(&upload.href)
                    .body(file_data)
                    .header("Content-Type", "application/octet-stream");
                
                if let Some(headers) = &upload.header {
                    for (k, v) in headers {
                        put_req = put_req.header(k, v);
                    }
                }

                let put_resp = put_req.send().await.map_err(|e| format!("LFS Upload Failed: {}", e))?;
                if !put_resp.status().is_success() {
                    return Err(format!("LFS PUT Error: {}", put_resp.status()));
                }
            }
        }
    }

    // 4. Create Pointer File Content
    let pointer_content = format!("version https://git-lfs.github.com/spec/v1\noid sha256:{}\nsize {}\n", oid, size);
    let pointer_base64 = general_purpose::STANDARD.encode(pointer_content);

    // 5. Commit Pointer File to Repo
    let path = format!("src/data/images/{}", file_name);
    let url = format!("{}/repos/{}/{}/contents/{}", API_BASE, owner, repo, path);
    
    let payload = UpdateFilePayload {
        message: format!("Add image {} (LFS)", file_name),
        content: pointer_base64,
        sha: None, // Always creating new file (for now)
        branch: "main".to_string(),
    };
    
    let resp = client.put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github.v3+json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;
        
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("GitHub Commit Failed: {} - {}", status, text));
    }
    
    // Return relative path for storage
    Ok(file_name)
}

pub async fn sync_orchids_to_github(orchids: Vec<Orchid>) -> Result<(), String> {
    let (token, owner, repo) = get_config().ok_or("GitHub configuration missing")?;
    let path = "src/data/orchids.json";
    let url = format!("{}/repos/{}/{}/contents/{}", API_BASE, owner, repo, path);
    
    let client = Client::new();
    
    // 1. Get current SHA
    let get_resp = client.get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| e.to_string())?;
        
    if !get_resp.status().is_success() {
        return Err(format!("Failed to fetch orchids.json: {}", get_resp.status()));
    }
    
    let file_info: GithubFile = get_resp.json().await.map_err(|e| e.to_string())?;
    
    // 2. Update content
    let new_content = serde_json::to_string_pretty(&orchids).map_err(|e| e.to_string())?;
    let new_content_base64 = general_purpose::STANDARD.encode(new_content);
    
    let payload = UpdateFilePayload {
        message: "Sync orchids from web app".to_string(),
        content: new_content_base64,
        sha: Some(file_info.sha),
        branch: "main".to_string(),
    };
    
    let put_resp = client.put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github.v3+json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;
        
    if !put_resp.status().is_success() {
        let text = put_resp.text().await.unwrap_or_default();
        return Err(format!("Failed to commit orchids.json: {}", text));
    }
    
    Ok(())
}
