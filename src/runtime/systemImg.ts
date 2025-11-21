import { invoke } from "@tauri-apps/api/core";

export interface ImageResult {
  path?: string;
  bytes: Uint8Array;
}

/**
 * 保存图片到系统相册 / Pictures
 */
export async function saveImage(bytes: Uint8Array | number[]): Promise<ImageResult> {
  const bytesArray = Array.isArray(bytes) ? bytes : Array.from(bytes);
  const result = await invoke("save_image", { bytes: bytesArray });
  return { path: result.path, bytes: new Uint8Array(result.bytes) };
}

/**
 * 读取图片
 * - Windows/Android: 单张用户选择
 * - Linux: 默认目录所有图片
 */
export async function loadImage(): Promise<ImageResult | ImageResult[]> {
  const result = await invoke("load_image");
  if (Array.isArray(result)) {
    return result.map((r: any) => ({
      path: r.path,
      bytes: new Uint8Array(r.bytes),
    }));
  } else {
    return {
      path: result.path,
      bytes: new Uint8Array(result.bytes),
    };
  }
}
