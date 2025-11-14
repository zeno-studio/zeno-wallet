use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use image::io::Reader as ImageReader;
use image::ImageOutputFormat;
use tauri::Runtime;

static ALLOWED_EXT: [&str; 2] = ["png"];

#[derive(Serialize)]
pub struct ImageInfo {
    pub filename: String,
    pub thumbnail: Vec<u8>,
}

#[derive(Serialize)]
pub struct ImageData {
    pub bytes: Vec<u8>,
}

#[derive(Serialize)]
pub struct SaveResult {
    pub filename: String,
}

// =======================
// 工具函数
// =======================
fn is_allowed(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|ext| ALLOWED_EXT.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn make_thumbnail(path: &Path, max_dim: u32) -> Option<Vec<u8>> {
    let img = ImageReader::open(path).ok()?.decode().ok()?;
    let thumb = img.thumbnail(max_dim, max_dim);
    let mut buf = vec![];
    thumb.write_to(&mut buf, ImageOutputFormat::Jpeg(70)).ok()?;
    Some(buf)
}

#[cfg(target_os = "android")]
fn make_thumbnail_from_bytes(bytes: &[u8], max_dim: u32) -> Option<Vec<u8>> {
    let img = image::load_from_memory(bytes).ok()?;
    let thumb = img.thumbnail(max_dim, max_dim);
    let mut buf = vec![];
    thumb.write_to(&mut buf, ImageOutputFormat::Jpeg(70)).ok()?;
    Some(buf)
}

// =======================
// Windows/Linux: 列出默认目录图片（不递归）
// =======================
#[cfg(any(target_os = "windows", target_os = "linux"))]
#[tauri::command]
pub async fn list_images() -> Result<Vec<ImageInfo>, String> {
    let picture_dir = tauri::api::path::picture_dir()
        .or_else(|| tauri::api::path::home_dir().map(|h| h.join("Pictures")))
        .ok_or("Cannot determine pictures directory")?;

    if !picture_dir.exists() {
        fs::create_dir_all(&picture_dir).map_err(|e| e.to_string())?;
        return Ok(vec![]);
    }

    let mut results = vec![];
    for entry in fs::read_dir(&picture_dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_file() && is_allowed(&path) {
            if let Some(thumbnail) = make_thumbnail(&path, 200) {
                results.push(ImageInfo {
                    filename: path.file_name().unwrap().to_string_lossy().to_string(),
                    thumbnail,
                });
            }
        }
    }
    Ok(results)
}

// =======================
// Android: 调用插件
// =======================
#[cfg(target_os = "android")]
use tauri::plugin::mobile::invoke as android_invoke;

#[cfg(target_os = "android")]
#[tauri::command]
pub async fn list_images() -> Result<Vec<ImageInfo>, String> {
    let res: serde_json::Value = android_invoke("photo", "listImages", serde_json::json!({}))?;
    let arr = res.as_array().ok_or("Invalid plugin response")?;

    let mut results = vec![];
    for item in arr {
        let filename = item["name"].as_str().unwrap_or("").to_string();
        let bytes: Vec<u8> = item["bytes"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|v| v.as_u64().unwrap() as u8)
            .collect();
        if !bytes.is_empty() {
            if let Some(thumbnail) = make_thumbnail_from_bytes(&bytes, 200) {
                results.push(ImageInfo { filename, thumbnail });
            }
        }
    }
    Ok(results)
}

// =======================
// 读取图片原始 bytes
// Windows/Linux: 默认目录 + 文件名
// Android: 调用插件按文件名
// =======================
#[tauri::command]
pub async fn read_image(filename: String) -> Result<ImageData, String> {
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        let picture_dir = tauri::api::path::picture_dir()
            .or_else(|| tauri::api::path::home_dir().map(|h| h.join("Pictures")))
            .ok_or("Cannot determine pictures directory")?;
        let full_path = picture_dir.join(&filename);
        let bytes = fs::read(full_path).map_err(|e| e.to_string())?;
        return Ok(ImageData { bytes });
    }

    #[cfg(target_os = "android")]
    {
        let res: serde_json::Value =
            android_invoke("photo", "readImageByName", serde_json::json!({ "name": filename }))?;
        let bytes: Vec<u8> = res["bytes"]
            .as_array()
            .ok_or("Invalid plugin response")?
            .iter()
            .map(|v| v.as_u64().unwrap() as u8)
            .collect();
        return Ok(ImageData { bytes });
    }
}

// =======================
// 保存图片
// =======================
#[cfg(any(target_os = "windows", target_os = "linux"))]
#[tauri::command]
pub async fn save_image(bytes: Vec<u8>) -> Result<SaveResult, String> {
    let picture_dir = tauri::api::path::picture_dir()
        .or_else(|| tauri::api::path::home_dir().map(|h| h.join("Pictures")))
        .ok_or("Cannot determine pictures directory")?;

    if !picture_dir.exists() {
        fs::create_dir_all(&picture_dir).map_err(|e| e.to_string())?;
    }

    let filename = format!("image_{}.png", chrono::Local::now().timestamp());
    let full_path = picture_dir.join(&filename);

    fs::write(&full_path, &bytes).map_err(|e| e.to_string())?;

    Ok(SaveResult {
        filename: filename.to_string(),
    })
}

#[cfg(target_os = "android")]
#[tauri::command]
pub async fn save_image(bytes: Vec<u8>) -> Result<SaveResult, String> {
    let result: serde_json::Value =
        android_invoke("photo", "saveToGallery", serde_json::json!({ "bytes": bytes }))?;
    Ok(SaveResult {
        filename: result["name"].as_str().unwrap_or("").to_string(),
    })
}
