use reqwest::{Client, Method, header};
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};
use crate::orchid::Orchid;
use gloo_storage::{LocalStorage, Storage};
use web_sys::{FileReader, Blob};
use wasm_bindgen::JsCast;
use gloo_file::File;

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
    
    // We upload to src/data/images so it's part of the source tree
    let path = format!("src/data/images/{}", file_name);
    let url = format!("{}/repos/{}/{}/contents/{}", API_BASE, owner, repo, path);
    
    let client = Client::new();
    let content_base64 = general_purpose::STANDARD.encode(&file_data);
    
    let payload = UpdateFilePayload {
        message: format!("Add image {}", file_name),
        content: content_base64,
        sha: None, // Always creating new file
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
        return Err(format!("GitHub Upload Failed: {} - {}", status, text));
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
