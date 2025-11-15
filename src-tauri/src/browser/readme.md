[main window] ──创建──► [dapp window] ← 极简权限
                          │
                          ├── shell.html（本地静态 HTML）
                          │     ├── 顶部 Nav: 余额 + 网络 + 关闭
                          │     ├── <webview> 标签加载 DApp
                          │     └── Modal: 交易确认 / 进度
                          │
                          └── JS 桥接：main ↔ shell ↔ DApp

1. tauri.conf.json：注册 DApp 窗口（非 main）
2. src-tauri/dapp-shell.html（本地静态壳）

关键说明（Rust 端）

sign_transaction 示例中我用了 simulate_sign，请在真实产品中替换为安全签名实现（例如把私钥放在受保护存储或使用硬件签名，且签名操作 必须 在 Rust 端完成，前端不能直接访问私钥）。

我用 app_handle.emit_all("ETH_RESPONSE", resp) 来把签名结果广播回所有窗口。你的 shell 窗口会 event.listen('ETH_RESPONSE', ...) 去接收并把内容 window.postMessage 回给 webview。也可以 emit_to 指定特定窗口。

强烈建议在 Rust 端维护 origin 白名单、请求限速、以及每次请求前弹出用户确认窗口（或硬件确认）。

4) 安全与最佳实践建议（精简）

不要信任 webview 中的 JS：所有安全决策（是否允许签名、是否允许跨域）必须由 Rust（后端）强制执行。前端的白名单只是 UX 辅助。

白名单 origin：在创建 dapp 窗口时把允许的 origin 列表传给 shell，并在 Rust 端对每个 WebviewEthRequest.origin 做严格检查。

最小权限模型：默认拒绝敏感 RPC（读取私钥、导出密钥、发送交易）除非显式允许并由用户确认。

时间限制与超时：provider 实现有超时（示例 2 分钟），避免挂起请求消耗资源。

事件通道：使用 Window.emit / event.listen 作为可靠 IPC 通道，避免直接把私钥或长数据通过不受控的 postMessage 泄露。

签名审计：在弹窗显示尽可能多的可读细节（to, value, data decode、nonce、gas price），帮助用户判断。

日志：在 Rust 端记录请求来源/时间/decision（注意记录不要泄露私钥或敏感数据），便于审计。


下面我把你的 APP_STATE（全局状态） 融入完整的 多 Dapp session + EIP-1193 Provider + 多链 + 事件机制 + EIP-712 签名 结构，整体保持：

AppState = WalletCore + UiConfig（你已有）

WalletCore 提供链、账户、签名 (eip1559_tx, sign_message, sign_typed_data)

UiConfig 管状态（当前链、当前账户、多窗口 session、弹窗等）

Rust 负责所有 provider 响应

支持：
✔ chainChanged
✔ accountsChanged
✔ 多链切换
✔ 多 Dapp 窗口 session
✔ eth_requestAccounts（含异步确认）
✔ eth_sendTransaction
✔ personal_sign
✔ eth_signTypedData_v4 (EIP-712)
✔ 兼容 MetaMask / Rabby 交互逻辑

代码组织清晰、可直接放进项目。

2. 关键防御点总览
威胁	防御技术
DApp 注入恶意 JS 读取本地文件	禁用 Tauri APIs、禁用 IPC、禁用 FS
DApp 伪造 provider / 劫持事件	使用 iframe 沙箱 + JSContext 隔离
XSS加载本地资源	URL 白名单、content-security-policy、阻止 file://
DApp 跳转伪造地址骗签	URL 监控、origin 绑定 session
DApp 刷屏请求签名	Rust 层限速 + Pending Map 去重
MITM 或恶意脚本	禁用 eval、禁止外部 script
