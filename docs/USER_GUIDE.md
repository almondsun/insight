# User Guide

insIGht organizes official Instagram relationship exports into accounts and snapshots. An account is a local history; each import into that history is an immutable snapshot. This guide describes current `main`; see [version differences](GETTING_STARTED.md#version-differences) for the published v0.1.1 workflow.

## Accounts and snapshots

Use the account selector to switch between local histories. Import a later export into the matching account to build a timeline. Use a separate account for an export belonging to a different Instagram username.

On current `main`, the import preview shows the source filename, follower total, following total, warnings, and archive owner. Confirm these details before importing. Current `main` rejects an owner username that conflicts with owner metadata in the archive. Published v0.1.1 does not include this owner-confirmation step.

Importing the same normalized follower and following membership twice is treated as a duplicate snapshot even if the ZIP filename or import time differs.

## Relationship categories

| Category | Meaning |
| --- | --- |
| Followers | Accounts present in the snapshot's follower files. |
| Following | Accounts present in the snapshot's following file. |
| Mutuals | Accounts that appear in both sets. |
| Not following back | Accounts you follow that are not in your follower set. |
| Followers you do not follow | Accounts in your follower set that you do not follow. |

Select a category and type part of a username in **Search username** to filter it. Usernames are matched without regard to letter case.

## Changes between imports

Open **Changes** after importing at least two snapshots into the same account. insIGht compares the newest snapshot with the previous snapshot and marks usernames as added or removed within the selected relationship category.

Instagram exports are snapshots, not event logs. insIGht can only establish that a relationship differed between two imports; it cannot determine the exact moment of the change. Instagram also does not reliably provide a stable numeric account identifier, so a username change can appear as one removal and one addition.

## Export a relationship list

Select a category, then choose **CSV** or **JSON**. The native save dialog lets you choose the destination. Reports contain the normalized relationship rows for the active snapshot and category.

The export buttons export the active relationship category, not the additions and removals shown in the Changes view. Change-report export is not currently available.

Treat reports as personal data: they may contain usernames and normalized Instagram profile URLs.

## Delete a snapshot

In **Import history**, choose the trash button beside a snapshot and confirm deletion. Deletion removes that snapshot and its relationships from the local database. It cannot be undone through insIGht, although you can import the source archive again if you still have it.

Deleting an imported snapshot does not delete or modify the original ZIP or extracted folder.

## What insIGht reads

From a complete export, insIGht selects only:

- `following.json`
- `followers.json` or numbered `followers_*.json` files
- owner metadata from the nested personal-information file when present

It extracts usernames, sanitized canonical profile URLs when available, and relationship timestamps when present. It does not copy the source archive into application storage.

## Current limitations

- Current `main` supports complete Instagram JSON ZIP exports and extracted export folders.
- Current `main` requires both follower and following relationship files; published v0.1.1 accepts partial and standalone JSON input.
- Account labels cannot currently be renamed in the interface.
- The interface deletes individual snapshots but does not currently expose full-account deletion.
- The local SQLite database is not encrypted by insIGht.
- Fame retrieval and ranking are not available in the user interface.
