# GitHub Repository Metadata

Status: applied and verified on 2026-08-11

This file is the source of truth for Nivune's public GitHub About copy and repository-discovery topics. The live values must be read back after every update and kept aligned with this file.

The canonical repository is `almondsun/nivune`. GitHub may redirect former repository URLs, but documentation and automation must use the canonical name.

## About

**Description**

> Local-first desktop analytics for official Instagram exports — followers, mutuals, history, and reports without login or cloud upload.

**Website**

<https://github.com/almondsun/nivune/releases/latest>

## Topics

```text
instagram
instagram-analytics
instagram-export
followers
follower-tracker
relationship-analytics
local-first
offline-first
privacy
privacy-tools
desktop-app
tauri
```

Topics prioritize the product, user problem, and privacy model over replaceable implementation dependencies. Fame is omitted because it is not a released feature.

## Repository features

- Issues: enabled
- Discussions: enabled
- Wiki: disabled
- Pages: disabled

Repository Markdown under `docs/` is the canonical documentation until a separately maintained documentation site exists.

## Applying the metadata

With an authenticated GitHub CLI session authorized for `almondsun/nivune`:

```bash
gh repo edit almondsun/nivune \
  --description "Local-first desktop analytics for official Instagram exports — followers, mutuals, history, and reports without login or cloud upload." \
  --homepage "https://github.com/almondsun/nivune/releases/latest"
```

Replace the topic set through the GitHub API:

```bash
gh api --method PUT repos/almondsun/nivune/topics \
  -f 'names[]=instagram' \
  -f 'names[]=instagram-analytics' \
  -f 'names[]=instagram-export' \
  -f 'names[]=followers' \
  -f 'names[]=follower-tracker' \
  -f 'names[]=relationship-analytics' \
  -f 'names[]=local-first' \
  -f 'names[]=offline-first' \
  -f 'names[]=privacy' \
  -f 'names[]=privacy-tools' \
  -f 'names[]=desktop-app' \
  -f 'names[]=tauri'
```

Read the repository metadata back and compare it with this file. Do not add a topic for an unreleased roadmap feature.
