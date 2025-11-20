# DApp Browser

DApp浏览器是一个基于Tauri的原生Webview浏览器，专为与区块链DApp交互而设计。

## 功能特性

- **安全沙箱**: 通过Content Security Policy (CSP) 限制DApp的访问权限
- **Ethereum Provider API**: 实现了兼容MetaMask的window.ethereum接口
- **签名请求处理**: 拦截并处理来自DApp的签名请求
- **多链支持**: 支持不同的区块链网络
- **账户管理**: 管理和切换不同的钱包账户
- **响应式UI**: 现代化的用户界面，支持暗色主题

## 架构设计

```
DApp Browser
├── HTML Shell (dapp-shell.html)
│   ├── WebView容器
│   ├── Ethereum Provider注入
│   └── 签名确认UI
├── Rust Backend (dapp.rs)
│   ├── Tauri命令处理
│   ├── 签名逻辑
│   └── 状态管理
└── 通信机制
    ├── Tauri IPC
    └── WebView消息传递
```

## 核心组件

### 1. HTML Shell (dapp-shell.html)

- 提供WebView容器加载DApp
- 注入Ethereum Provider API
- 处理签名请求的UI确认
- 主题和状态管理

### 2. Rust Backend (dapp.rs)

- 实现Tauri命令接口
- 处理签名和账户管理
- 与主应用状态交互
- 安全检查和权限控制

## API接口

### Tauri Commands

- `get_darkmode()`: 获取当前主题模式
- `close_dapp_window()`: 关闭DApp窗口
- `get_balance()`: 获取账户余额
- `sign_transaction()`: 处理签名请求
- `open_dapp_window()`: 打开新的DApp窗口
- `get_current_address()`: 获取当前账户地址
- `get_chain_id()`: 获取当前链ID

### Ethereum Provider Methods

- `ethereum.request()`: 发送JSON-RPC请求
- `ethereum.on()`: 事件监听
- `ethereum.removeListener()`: 移除事件监听

## 安全特性

1. **Content Security Policy**: 限制脚本和资源加载
2. **Origin检查**: 验证DApp来源
3. **用户确认**: 所有敏感操作需要用户确认
4. **沙箱隔离**: WebView与主应用隔离

## 使用方法

1. 通过`open_dapp_window`命令打开DApp
2. DApp通过`window.ethereum`与钱包交互
3. 签名请求会弹出确认对话框
4. 用户确认后完成签名操作

## 开发指南

### 添加新的Ethereum方法

1. 在HTML Shell中扩展Ethereum Provider实现
2. 在Rust Backend中添加对应的处理逻辑
3. 注册新的Tauri命令

### 自定义UI

- 修改CSS样式调整界面外观
- 添加新的模态框处理特定请求类型
- 扩展状态显示信息

## 注意事项

- 不要在生产环境中使用模拟签名功能
- 确保所有敏感操作都有用户确认步骤
- 定期更新安全策略和权限检查