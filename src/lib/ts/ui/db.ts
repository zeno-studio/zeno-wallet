// src/db.ts
import Database from '@tauri-apps/plugin-sql';

let db: Database;

export async function initDB(password: string) {
  db = await Database.load('sqlite:wallet.db');
  await db.execute('PRAGMA key = ?', [password]); // SQLCipher 加密
  await db.execute(`
    CREATE TABLE IF NOT EXISTS kv (
      key TEXT PRIMARY KEY,
      value TEXT,
      updated_at INTEGER
    )
  `);
}

// 通用读写
export async function set(key: string, value: any) {
  const json = JSON.stringify(value);
  await db.execute(
    'INSERT INTO kv (key, value, updated_at) VALUES (?, ?, ?) ON CONFLICT(key) DO UPDATE SET value = ?, updated_at = ?',
    [key, json, Date.now(), json, Date.now()]
  );
}

export async function get<T>(key: string, defaultValue: T): Promise<T> {
  const rows = await db.select<{ value: string }[]>(
    'SELECT value FROM kv WHERE key = ?',
    [key]
  );
  return rows[0] ? JSON.parse(rows[0].value) : defaultValue;
}