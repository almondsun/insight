import { invoke } from "@tauri-apps/api/core";

export type Account={id:number;label:string;username:string|null;snapshotCount:number};
export type Snapshot={id:number;accountId:number;importedAt:string;observedAt:string;observedAtSource:"user_confirmed"|"legacy_import_time";sourceName:string;followers:number;following:number};
export type Summary={followers:number;following:number;mutuals:number;notFollowingBack:number;followersNotFollowedBack:number;newFollowers:number;lostFollowers:number;hasPreviousSnapshot:boolean};
export type Relationship={username:string;profileUrl:string|null;kind:string};
export type Change={username:string;profileUrl:string|null;category:string;direction:"added"|"removed"};
export type RelationshipPage={items:Relationship[];nextCursor:string|null};
export type ChangePage={items:Change[];nextCursor:string|null};
export type ImportPreview={token:string;sourceName:string;detectedUsername:string|null;followers:number;following:number;warnings:string[]};
export type TrendPoint={snapshotId:number;observedAt:string;followers:number;following:number;mutuals:number;newFollowers:number;lostFollowers:number};
export type RelationshipHistoryPoint={snapshotId:number;observedAt:string;followsYou:boolean;youFollow:boolean};
export type FameFoundationStatus={implementationStage:"synthetic_foundation";formulaVersion:string;protocolSchemaVersion:number;fixedCorpusRecordBytes:number;networkRetrievalAvailable:false;architectureStatus:"frozen";nextStage:string;completedFoundations:string[];blockedGates:string[]};

export const api={
  accounts:()=>invoke<Account[]>("list_accounts"), snapshots:(accountId:number)=>invoke<Snapshot[]>("list_snapshots",{accountId}),
  summary:(accountId:number,snapshotId?:number)=>invoke<Summary>("get_summary",{accountId,snapshotId}),
  trends:(accountId:number)=>invoke<TrendPoint[]>("get_trends",{accountId}),
  relationshipHistory:(accountId:number,username:string)=>invoke<RelationshipHistoryPoint[]>("get_relationship_history",{accountId,username}),
  relationships:(snapshotId:number,kind:string,search:string,after:string|null=null,limit=200)=>invoke<RelationshipPage>("get_relationships",{snapshotId,kind,search,after,limit}),
  changes:(fromSnapshotId:number,toSnapshotId:number,category:string,search:string,after:string|null=null,direction:string|null=null,limit=200)=>invoke<ChangePage>("compare_snapshots",{fromSnapshotId,toSnapshotId,category,search,after,direction,limit}),
  renameAccount:(accountId:number,label:string)=>invoke<Account>("rename_account",{accountId,label}),
  fameFoundationStatus:()=>invoke<FameFoundationStatus>("get_fame_foundation_status"),
  chooseImport:(directory:boolean)=>invoke<ImportPreview|null>("choose_import",{directory}),
  commit:(token:string,accountId:number|null,label:string,ownerUsername:string,observedAt:string)=>invoke<Snapshot>("commit_import",{token,accountId,label,ownerUsername,observedAt}),
  cancelImport:(token:string)=>invoke<void>("cancel_import",{token}),
  deleteSnapshot:(snapshotId:number)=>invoke<void>("delete_snapshot",{snapshotId}), deleteAccount:(accountId:number)=>invoke<void>("delete_account",{accountId}),
  exportReport:(snapshotId:number,kind:string,format:string)=>invoke<boolean>("export_report",{snapshotId,kind,format}),
  exportChanges:(fromSnapshotId:number,toSnapshotId:number,category:string,direction:string|null,format:string)=>invoke<boolean>("export_changes",{fromSnapshotId,toSnapshotId,category,direction,format}),
  createEncryptedBackup:(passphrase:string)=>invoke<boolean>("create_encrypted_backup",{passphrase}),
  restoreEncryptedBackup:(passphrase:string)=>invoke<boolean>("restore_encrypted_backup",{passphrase})
};

export type NivuneApi = typeof api;
