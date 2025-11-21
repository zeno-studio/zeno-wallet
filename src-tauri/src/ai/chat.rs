#[tauri::command]
async fn ai_ask(query: String, context: serde_json::Value) -> Result<String, String> {
    // 调用本地 Ollama（完全离线）或 Groq（免费 tier）
    let prompt = format!(
        "你是一个只读助手，只能建议，不能执行交易。\n当前上下文：{}\n用户问：{}\n如果建议小额支付（<0.01 ETH），在回复末尾加【建议小额支付】",
        serde_json::to_string_pretty(&context).unwrap(),
        query
    );
    
    let client = reqwest::Client::new();
    let res = client.post("http://localhost:11434/api/generate")  // Ollama 本地
        .json(&json!({
            "model": "llama3.2:3b",
            "prompt": prompt,
            "stream": false
        }))
        .send().await.unwrap()
        .json::<serde_json::Value>().await.unwrap();
    
    Ok(res["response"].as_str().unwrap().to_string())
}


