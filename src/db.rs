use rexie::{Rexie, ObjectStore};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Blob;
use crate::error::AppError;

const DB_NAME: &str = "OrchidImagesDB";
const STORE_NAME: &str = "images";

pub async fn init_db() -> Result<Rexie, AppError> {
    Rexie::builder(DB_NAME)
        .version(1)
        .add_object_store(
            ObjectStore::new(STORE_NAME)
                .auto_increment(true)
        )
        .build()
        .await
        .map_err(|e| AppError::Database(e.to_string()))
}

pub async fn save_image_blob(blob: Blob) -> Result<u32, AppError> {
    let rexie = init_db().await?;

    let transaction = rexie
        .transaction(&[STORE_NAME], rexie::TransactionMode::ReadWrite)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let store = transaction.store(STORE_NAME)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let id_js = store.add(&JsValue::from(blob), None).await
        .map_err(|e| AppError::Database(e.to_string()))?;

    transaction.done().await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let id = id_js.as_f64()
        .ok_or_else(|| AppError::Database("Failed to parse ID as number".into()))? as u32;

    Ok(id)
}

pub async fn get_image_blob(id: u32) -> Result<Option<Blob>, AppError> {
    let rexie = init_db().await?;

    let transaction = rexie
        .transaction(&[STORE_NAME], rexie::TransactionMode::ReadOnly)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let store = transaction.store(STORE_NAME)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let key = JsValue::from_f64(id as f64);
    let result_opt = store.get(key).await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if let Some(result_js) = result_opt {
        if result_js.is_undefined() || result_js.is_null() {
            return Ok(None);
        }
        let blob = result_js.dyn_into::<Blob>()
            .map_err(|_| AppError::Database("Failed to cast to Blob".into()))?;
        Ok(Some(blob))
    } else {
        Ok(None)
    }
}
