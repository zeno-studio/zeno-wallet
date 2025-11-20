AI-Chat WebView（纯静态 HTML + inline JS）
        │
        ├─► 用户输入文字 → invoke("ai_ask", { text })
        │
        └─► Rust 收到 → 调用 piper-rs 本地合成 → 得到 PCM 流
                 │
                 ├─► 把 PCM 切成小块（每 20~50ms 一块）
                 │
                 └─► 通过 tauri::event.emit("tts-chunk", chunk: Vec<u8>) 广播
                          │
                          ▼
               AI-Chat WebView 监听 "tts-chunk"
                          │
               把收到的 chunk 拼进 AudioBuffer → Web Audio API 实时播放


// TTS 示例
let piper = Piper::new("resources/voices/zh_CN-huayan-medium.onnx")?;

// STT 示例
let ctx = WhisperContext::new_with_params("resources/models/ggml-tiny.bin", params)?;


stt
https://github.com/ggml-org/whisper.cpp/tree/master/models  

初始包只内置：STT：一个 ggml-tiny.bin（75 MB）→ 全球 99 语种语音输入
TTS：一个英文 medium/low 模型（35～45 MB）→ 兜底语音输出

总增量 ≈ 110～120 MB
加上你原来的钱包本体（15～20 MB）+ 其他资源，最终安装包轻松控制在 130～150 MB，和现在主流桌面钱包一模一样（Rabby v1.8 就是 142 MB，Phantom 桌面版 138 MB）。其他语种 TTS 完全按需下载（用户第一次要点“用中文朗读”时才下）

一行命令下载 tiny 全语种模型（直接可用于 whisper-rs）

# 方法 1：官方脚本（最简单）
bash <(curl -sSL https://raw.githubusercontent.com/ggerganov/whisper.cpp/master/models/download-ggml-model.sh) tiny

# 方法 2：直链（放进你的 resources/models/）
wget -O resources/models/ggml-tiny.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin