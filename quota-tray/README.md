# Quota Tray

Local-only multi-provider AI quota tray (Tauri 2 + Rust + Svelte).

## Develop

```bash
cd quota-tray/ui && npm install
cd ../src-tauri && cargo tauri dev
```

## Test Rust crates

```bash
cd quota-tray && cargo test -p quota_tray_core -p provider_claude
```

Credentials never leave the machine. No Quota Tray servers. Vendor usage APIs may be unofficial.
