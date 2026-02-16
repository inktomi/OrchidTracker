use reqwest::Client;
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};
use crate::orchid::Orchid;
use crate::error::AppError;
use gloo_storage::{LocalStorage, Storage};
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize, Debug)]
struct GithubFile {
    sha: String,
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

fn get_config() -> Result<(String, String, String), AppError> {
    let token: String = LocalStorage::get("github_token").map_err(|_| AppError::ConfigMissing)?;
    let owner: String = LocalStorage::get("repo_owner").map_err(|_| AppError::ConfigMissing)?;
    let repo: String = LocalStorage::get("repo_name").map_err(|_| AppError::ConfigMissing)?;
    if token.is_empty() || owner.is_empty() || repo.is_empty() {
        return Err(AppError::ConfigMissing);
    }
    Ok((token, owner, repo))
}

pub async fn upload_image_to_github(file_name: String, file_data: Vec<u8>) -> Result<String, AppError> {
    let (token, owner, repo) = get_config()?;
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
        operation: "upload".into(),
        transfers: vec!["basic".into()],
        objects: vec![LfsObject { oid: oid.clone(), size }],
    };

    let batch_resp = client.post(&lfs_url)
        .basic_auth("git", Some(&token))
        .header("Accept", "application/vnd.git-lfs+json")
        .header("Content-Type", "application/vnd.git-lfs+json")
        .json(&batch_req)
        .send()
        .await
        .map_err(|e| AppError::LfsUpload(format!("Batch init failed: {}", e)))?;

    if !batch_resp.status().is_success() {
        return Err(AppError::LfsUpload(format!("Init error: {}", batch_resp.status())));
    }

    let batch_data: LfsBatchResponse = batch_resp.json().await
        .map_err(|e| AppError::LfsUpload(e.to_string()))?;

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

                let put_resp = put_req.send().await
                    .map_err(|e| AppError::LfsUpload(format!("Upload failed: {}", e)))?;
                if !put_resp.status().is_success() {
                    return Err(AppError::LfsUpload(format!("PUT error: {}", put_resp.status())));
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
        sha: None,
        branch: "main".into(),
    };

    let resp = client.put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github.v3+json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::GithubApi(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::GithubApi(format!("Commit failed: {} - {}", status, text)));
    }

    Ok(file_name)
}

pub async fn sync_orchids_to_github(orchids: Vec<Orchid>) -> Result<(), AppError> {
    let (token, owner, repo) = get_config()?;
    let path = "src/data/orchids.json";
    let url = format!("{}/repos/{}/{}/contents/{}", API_BASE, owner, repo, path);

    let client = Client::new();

    // 1. Get current SHA
    let get_resp = client.get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    if !get_resp.status().is_success() {
        return Err(AppError::GithubApi(format!("Failed to fetch orchids.json: {}", get_resp.status())));
    }

    let file_info: GithubFile = get_resp.json().await
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    // 2. Update content
    let new_content = serde_json::to_string_pretty(&orchids)
        .map_err(|e| AppError::Serialization(e.to_string()))?;
    let new_content_base64 = general_purpose::STANDARD.encode(new_content);

    let payload = UpdateFilePayload {
        message: "Sync orchids from web app".into(),
        content: new_content_base64,
        sha: Some(file_info.sha),
        branch: "main".into(),
    };

    let put_resp = client.put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github.v3+json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    if !put_resp.status().is_success() {
        let text = put_resp.text().await.unwrap_or_default();
        return Err(AppError::GithubApi(format!("Failed to commit orchids.json: {}", text)));
    }

    Ok(())
}
