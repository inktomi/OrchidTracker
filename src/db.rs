use rexie::{Rexie, ObjectStore};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Blob;

const DB_NAME: &str = "OrchidImagesDB";
const STORE_NAME: &str = "images";

// Helper function to initialize the database
pub async fn init_db() -> Result<Rexie, rexie::Error> {
    let rexie = Rexie::builder(DB_NAME)
        .version(1)
        .add_object_store(
            ObjectStore::new(STORE_NAME)
                .auto_increment(true)
        )
        .build()
        .await?;
    Ok(rexie)
}

// Function to save an image (Blob) to IndexedDB and return its ID
pub async fn save_image_blob(blob: Blob) -> Result<u32, String> {
    let rexie = init_db().await.map_err(|e| e.to_string())?;

    // Create a read-write transaction
    let transaction = rexie
        .transaction(&[STORE_NAME], rexie::TransactionMode::ReadWrite)
        .map_err(|e| e.to_string())?;

    let store = transaction.store(STORE_NAME).map_err(|e| e.to_string())?;

    // Add the blob. The second argument is the key, which is None because we use auto_increment.
    // The `add` method returns the key (ID) of the inserted item.
    let id_js = store.add(&JsValue::from(blob), None).await.map_err(|e| e.to_string())?;
    
    // Commit the transaction
    transaction.done().await.map_err(|e| e.to_string())?;

    // The returned key should be a number (the auto-incremented ID)
    let id = id_js.as_f64().ok_or("Failed to parse ID as number")? as u32;

    Ok(id)
}

// Function to retrieve an image (Blob) from IndexedDB by ID
pub async fn get_image_blob(id: u32) -> Result<Option<Blob>, String> {
    let rexie = init_db().await.map_err(|e| e.to_string())?;

    let transaction = rexie
        .transaction(&[STORE_NAME], rexie::TransactionMode::ReadOnly)
        .map_err(|e| e.to_string())?;

    let store = transaction.store(STORE_NAME).map_err(|e| e.to_string())?;

    let key = JsValue::from_f64(id as f64);
    let result_opt = store.get(key).await.map_err(|e| e.to_string())?;
    
    if let Some(result_js) = result_opt {
        if result_js.is_undefined() || result_js.is_null() {
            return Ok(None);
        }
        // Cast JsValue to Blob
        let blob = result_js.dyn_into::<Blob>().map_err(|_| "Failed to cast to Blob")?;
        Ok(Some(blob))
    } else {
        Ok(None)
    }
}
