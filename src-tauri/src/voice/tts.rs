use piper_rs::Piper;
use tauri::Manager;

#[tauri::command]
async fn ai_ask(text: String, app: tauri::AppHandle) -> Result<(), String> {
    let model_path = "voices/zh_CN-huayan-medium.onnx";
    let mut piper = Piper::new(model_path).map_err(|e| e.to_string())?;

    // 流式合成（piper-rs 支持 stream）
    let mut stream = piper.synthesize_stream(text).map_err(|e| e.to_string())?;

    // 每来一块 PCM 就广播出去
    while let Some(chunk) = stream.next() {
        // chunk 是 Vec<i16> PCM（16-bit 22.05kHz）
        let bytes: Vec<u8> = chunk
            .iter()
            .flat_map(|&sample| sample.to_le_bytes().to_vec())
            .collect();

        app.emit_all("tts-chunk", bytes).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20)); // 控制码率
    }

    // 结束信号
    app.emit_all("tts-end", ()).unwrap();
    Ok(())
}