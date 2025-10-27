// src/utils/scanner.ts
import { scan, Format } from '@tauri-apps/plugin-barcode-scanner';
import { invoke } from '@tauri-apps/api/core';
import Html5QrcodeScanner from 'html5-qrcode';

let scanner: Html5QrcodeScanner | null = null;

export async function startScan(formats: string[] = ['QR_CODE']): Promise<{ content: string; format: string }> {
  try {
    // 检测平台（Tauri API）
    const platform = await invoke<string>('get_platform'); // Rust 命令返回 'android'/'ios'/'windows' 等

    if (['android', 'ios'].includes(platform)) {
      // 移动端：用插件（类原生）
      const result = await scan({
        cameraDirection: 'back',
        formats: formats as Format[],
        windowed: true // 透明叠加到 UI
      });
      return { content: result.content, format: result.format };
    } else {
      // 桌面端：用 TS 库
      return new Promise((resolve, reject) => {
        scanner = new Html5QrcodeScanner('scanner-container', { fps: 10, qrbox: { width: 250, height: 250 } }, false);
        scanner.render(
          (decodedText, decodedResult) => {
            scanner?.clear(); // 扫描后停止
            resolve({ content: decodedText, format: decodedResult?.format?.name || 'QR_CODE' });
          },
          (error) => reject(error)
        );
      });
    }
  } catch (error) {
    console.error('扫描失败:', error);
    throw error;
  }
}

export function stopScan() {
  scanner?.clear();
}