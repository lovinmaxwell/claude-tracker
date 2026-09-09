/// <reference types="vite/client" />
/// <reference types="chrome" />

interface ImportMetaEnv {
  readonly QUOTA_TRAY_PLATFORM?: "chrome" | "tauri";
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
