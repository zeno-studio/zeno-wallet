#[tauri::command]
#[cfg(target_os = "ios")]
pub fn scan_qr() -> Result<String, String> {
    // Windows: 监听键盘事件模拟条码枪
    Ok("1234567890".to_string())
}

#[tauri::command]
#[cfg(not(target_os = "android"))]
pub fn scan_qr() -> Result<String, String> {
    Err("仅 Windows 支持 USB 条码枪".to_string())
}
