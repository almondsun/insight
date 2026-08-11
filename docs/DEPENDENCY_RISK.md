# Dependency Risk Register

This register records accepted prerelease dependency risk and its GitHub disposition without treating an unpatched dependency as fixed.

## `glib` GHSA-wrw7-89jp-8q8g

Status: unpatched and monitored; GitHub alert dismissed as vulnerable code not used

Severity: moderate

Reviewed and disposition recorded: 2026-08-11

GitHub reports an unsound `VariantStrIter` implementation in `glib` 0.18.5. The fixed line begins at 0.20, while Tauri 2.11.5's Linux GTK3 dependency graph currently resolves through `gtk` 0.18 to `glib` 0.18.5.

The relevant paths reported by `cargo tree -i glib@0.18.5` are:

```text
nivune -> tauri -> gtk -> glib 0.18.5
nivune -> tauri -> tauri-runtime-wry -> webkit2gtk/wry -> gtk -> glib 0.18.5
```

A static Rust-source search across Nivune, its workspace crates, tools, and the locally resolved Cargo registry found `VariantStrIter` and `array_iter_str` only inside the `glib` crate's implementation, documentation, and tests. It found no call in Nivune, Tauri, Wry, Tao, GTK, WebKitGTK, or the other resolved consumers. This reduces the evidence of reachability but does not prove the vulnerable code can never be reached at runtime.

Decision:

- Dismiss the GitHub alert as `not_used` because the affected iterator API has no identified caller in Nivune or its resolved consumers. The dismissal is an auditable triage decision, not a claim that `glib` 0.18.5 is patched.
- Do not vendor an unaudited patch or force an incompatible `glib` major version.
- Track Tauri/GTK upstream migration and update when the supported dependency graph permits it.
- Reassess before a stable release, after relevant dependency changes, or if new reachability evidence appears; reopen the alert if the affected API becomes reachable.

Reproduce the local evidence with:

```bash
cargo tree -i glib@0.18.5
rg -n "VariantStrIter|array_iter_str" src src-tauri crates tools ~/.cargo/registry/src --glob '*.rs'
```
