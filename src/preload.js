// @ts-nocheck
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// runs in privileged preload context
const allowedOrigins = new Set([
  // optionally add whitelisted origins you expect to load, or leave flexible with checks in Rust
  "https://app.uniswap.org",
  "https://app.somedapp.example"
]);

// Generate a unique ID for each request
let requestIdCounter = 0;
function generateRequestId() {
  return `req_${Date.now()}_${++requestIdCounter}`;
}

// Helper to send to Rust (using Tauri 2.x API)
async function sendToRust(method, params = {}) {
  try {
    const result = await invoke("wallet_request", {
      id: generateRequestId(),
      origin: window.location.origin,
      method,
      params
    });
    return result;
  } catch (err) {
    throw new Error(`Rust error: ${err}`);
  }
}

// Listen for messages from the DApp page (postMessage API)
window.addEventListener("message", async (ev) => {
  const msg = ev.data;
  if (!msg || !msg._walletReq) return;
  const origin = ev.origin || ev.source?.origin || null;

  // Optionally enforce origin whitelist client-side (server-side / Rust will re-check)
  if (origin && !allowedOrigins.has(origin)) {
    // reply with error to page
    if (ev.source) {
      ev.source.postMessage({ 
        _walletRespId: msg.id, 
        error: "origin_not_allowed" 
      }, origin);
    }
    return;
  }

  // Forward sanitized request to Rust
  try {
    const res = await invoke("wallet_request", {
      id: msg.id || generateRequestId(),
      origin: origin || window.location.origin,
      method: msg.method,
      params: msg.params || []
    });
    if (ev.source) {
      ev.source.postMessage({ 
        _walletRespId: msg.id, 
        result: res 
      }, origin);
    }
  } catch (err) {
    if (ev.source) {
      ev.source.postMessage({ 
        _walletRespId: msg.id, 
        error: String(err) 
      }, origin);
    }
  }
});

// EIP-1193 Provider implementation
// Injected into page context (preload runs before page load, so this will be available)
const ethereumProvider = {
  isMetaMask: false,
  isWallet: true,
  isTauriWallet: true,
  
  // EIP-1193 required methods
  request: async ({ method, params = [] }) => {
    try {
      return await sendToRust(method, params);
    } catch (err) {
      throw {
        code: -32603,
        message: err.message || String(err)
      };
    }
  },

  // Legacy Ethereum provider methods for compatibility
  send: async (method, params) => {
    return ethereumProvider.request({ method, params });
  },

  sendAsync: (payload, callback) => {
    ethereumProvider.request({ 
      method: payload.method, 
      params: payload.params || [] 
    })
      .then(result => callback(null, { id: payload.id, jsonrpc: "2.0", result }))
      .catch(err => callback(err, { id: payload.id, jsonrpc: "2.0", error: err }));
  },

  // Event emitter interface (minimal implementation)
  _events: {},
  on: function(event, handler) {
    if (!this._events[event]) this._events[event] = [];
    this._events[event].push(handler);
  },
  removeListener: function(event, handler) {
    if (this._events[event]) {
      this._events[event] = this._events[event].filter(h => h !== handler);
    }
  },
  emit: function(event, ...args) {
    if (this._events[event]) {
      this._events[event].forEach(handler => handler(...args));
    }
  },

  // Chain ID and accounts (will be populated when connected)
  chainId: null,
  selectedAddress: null,
  accounts: null
};

// Listen for Rust events and update provider state
listen("wallet:chainChanged", (event) => {
  ethereumProvider.chainId = event.payload.chainId;
  ethereumProvider.emit("chainChanged", event.payload.chainId);
});

listen("wallet:accountsChanged", (event) => {
  ethereumProvider.accounts = event.payload.accounts;
  ethereumProvider.selectedAddress = event.payload.accounts?.[0] || null;
  ethereumProvider.emit("accountsChanged", event.payload.accounts);
});

// Inject provider into window object
// Note: In Tauri 2.x, preload runs in a context where we can directly set window properties
window.ethereum = ethereumProvider;

// Also expose via globalThis for maximum compatibility
if (typeof globalThis !== "undefined") {
  globalThis.ethereum = ethereumProvider;
}