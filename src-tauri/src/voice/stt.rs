use whisper_rs::{WhisperContext, WhisperContextParameters};

static mut WHISPER: Option<WhisperContext> = None;

#[tauri::command]
fn init_whisper() {
    let ctx = WhisperContext::new_with_params(
        "models/ggml-base.bin",  // 或 ggml-tiny.bin（5MB）
        WhisperContextParameters::default(),
    ).unwrap();
    unsafe { WHISPER = Some(ctx); }
}

#[tauri::command]
fn transcribe_audio(audio_i16: Vec<i16>, app: tauri::AppHandle) -> String {
    let ctx = unsafe { WHISPER.as_ref().unwrap() };
    let mut state = ctx.create_state().unwrap();
    
    // 实时流式识别（Whisper.cpp 支持）
    state.full(audio_i16.as_slice()).unwrap();
    let mut result = String::new();
    while state.full_get_segment_text(0).is_ok() {
        result.push_str(state.full_get_segment_text(0).unwrap().as_str());
    }
    
    // 把识别结果发回 WebView 显示
    app.emit_all("stt-result", result.clone()).unwrap();
    result
}