use reqwest::Client;
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};
use crate::orchid::Orchid;
use crate::error::AppError;
use gloo_storage::{LocalStorage, Storage};
use sha2::{Sha256, Digest};

#[derive(Deserialize, Debug)]
struct GithubFileWithContent {
    sha: String,
    content: Option<String>,
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
    upload_image_via_lfs(file_name, file_data).await
}

async fn upload_image_via_lfs(file_name: String, file_data: Vec<u8>) -> Result<String, AppError> {
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
        .basic_auth(owner.as_str(), Some(&token))
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

/// Fetch remote orchids.json from GitHub. Returns (orchids, sha).
/// Returns (vec![], None) if the file doesn't exist yet (404).
async fn fetch_remote_orchids(
    client: &Client,
    token: &str,
    owner: &str,
    repo: &str,
) -> Result<(Vec<Orchid>, Option<String>), AppError> {
    let path = "src/data/orchids.json";
    let url = format!("{}/repos/{}/{}/contents/{}", API_BASE, owner, repo, path);

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    if resp.status() == 404 {
        return Ok((vec![], None));
    }

    if !resp.status().is_success() {
        return Err(AppError::GithubApi(format!(
            "Failed to fetch orchids.json: {}",
            resp.status()
        )));
    }

    let file_info: GithubFileWithContent = resp
        .json()
        .await
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    let content_b64 = file_info.content.unwrap_or_default();
    // GitHub's base64 embeds newlines — strip them before decoding
    let cleaned: String = content_b64.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = general_purpose::STANDARD
        .decode(cleaned)
        .map_err(|e| AppError::Serialization(format!("base64 decode: {}", e)))?;

    let orchids: Vec<Orchid> = serde_json::from_slice(&bytes)
        .map_err(|e| AppError::Serialization(format!("JSON parse: {}", e)))?;

    Ok((orchids, Some(file_info.sha)))
}

/// Push orchids as JSON to GitHub. Returns GithubConflict on 409.
async fn push_orchids(
    client: &Client,
    token: &str,
    owner: &str,
    repo: &str,
    orchids: &[Orchid],
    sha: Option<String>,
) -> Result<(), AppError> {
    let path = "src/data/orchids.json";
    let url = format!("{}/repos/{}/{}/contents/{}", API_BASE, owner, repo, path);

    let json_content = serde_json::to_string_pretty(orchids)
        .map_err(|e| AppError::Serialization(e.to_string()))?;
    let content_base64 = general_purpose::STANDARD.encode(json_content);

    let payload = UpdateFilePayload {
        message: "Sync orchids from web app".into(),
        content: content_base64,
        sha,
        branch: "main".into(),
    };

    let resp = client
        .put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github.v3+json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    if resp.status() == 409 {
        return Err(AppError::GithubConflict);
    }

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::GithubApi(format!(
            "Failed to commit orchids.json: {}",
            text
        )));
    }

    Ok(())
}

/// Bidirectional sync: pull remote, merge with local, push merged result.
/// Retries up to 3 times on conflict (409).
/// Returns the merged orchid list on success.
pub async fn bidirectional_sync(local_orchids: Vec<Orchid>) -> Result<Vec<Orchid>, AppError> {
    use crate::merge::merge_orchids;

    let (token, owner, repo) = get_config()?;
    let client = Client::new();

    for _ in 0..3 {
        let (remote, sha) = fetch_remote_orchids(&client, &token, &owner, &repo).await?;
        let merged = merge_orchids(local_orchids.clone(), remote);

        match push_orchids(&client, &token, &owner, &repo, &merged, sha).await {
            Ok(()) => return Ok(merged),
            Err(AppError::GithubConflict) => {
                log::warn!("Sync conflict, retrying...");
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    Err(AppError::GithubApi(
        "Sync failed after 3 retries due to conflicts".into(),
    ))
}
