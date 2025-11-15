import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";

let providerConfig = {
  chainId: null,
  accounts: [],
  selectedAddress: null,
};

window.ethereum = {
  isMetaMask: false,
  isRabby: false,
  isTauriWallet: true,

  request: async ({ method, params }) => {
    const res = await invoke("ethereum_request", {
      method,
      params,
    });
    return res;
  },

  on(event, handler) {
    listen(event, (e) => handler(e.payload));
  },
};

// Rust 下发 provider 信息
listen("wallet:provider-config", (e) => {
  providerConfig = e.payload;
});

// listen("wallet:connect", () => {
//   provider.onconnect && provider.onconnect({ chainId: provider.chainId });
// });

// listen("wallet:disconnect", () => {
//   provider.ondisconnect && provider.ondisconnect();
// });

// listen("chainChanged", (e) => {
//   provider.on('chainChanged', handler => handler(e.payload));
// });

// listen("accountsChanged", (e) => {
//   provider.on('accountsChanged', handler => handler(e.payload));
// });

window.eval = () => {
  throw new Error("Eval blocked by wallet");
};
window.Function = () => {
  throw new Error("Function constructor blocked");
};
