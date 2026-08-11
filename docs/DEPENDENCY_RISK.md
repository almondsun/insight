# Dependency Risk Register

This register records accepted prerelease dependency risk without treating an open alert as fixed or dismissing it for dashboard appearance.

## `glib` GHSA-wrw7-89jp-8q8g

Status: open, monitored, accepted for Nivune v0.2.0-preview.4 only

Severity: moderate

Reviewed: 2026-08-11

GitHub reports an unsound `VariantStrIter` implementation in `glib` 0.18.5. The fixed line begins at 0.20, while Tauri 2.11.5's Linux GTK3 dependency graph currently resolves through `gtk` 0.18 to `glib` 0.18.5.

The relevant paths reported by `cargo tree -i glib@0.18.5` are:

```text
nivune -> tauri -> gtk -> glib 0.18.5
nivune -> tauri -> tauri-runtime-wry -> webkit2gtk/wry -> gtk -> glib 0.18.5
```

A static Rust-source search across Nivune, its workspace crates, tools, and the locally resolved Cargo registry found `VariantStrIter` and `array_iter_str` only inside the `glib` crate's implementation, documentation, and tests. It found no call in Nivune, Tauri, Wry, Tao, GTK, WebKitGTK, or the other resolved consumers. This reduces the evidence of reachability but does not prove the vulnerable code can never be reached at runtime.

Decision:

- Keep the Dependabot alert open.
- Do not vendor an unaudited patch or force an incompatible `glib` major version.
- Track Tauri/GTK upstream migration and update when the supported dependency graph permits it.
- Reassess before a stable release, after relevant dependency changes, or if new reachability evidence appears.

Reproduce the local evidence with:

```bash
cargo tree -i glib@0.18.5
rg -n "VariantStrIter|array_iter_str" src src-tauri crates tools ~/.cargo/registry/src --glob '*.rs'
```
