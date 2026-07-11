import { invoke } from "@tauri-apps/api/core";

export type Account={id:number;label:string;username:string|null;snapshotCount:number};
export type Snapshot={id:number;accountId:number;importedAt:string;sourceName:string;followers:number;following:number};
export type Summary={followers:number;following:number;mutuals:number;notFollowingBack:number;followersNotFollowedBack:number;newFollowers:number;lostFollowers:number};
export type Relationship={username:string;profileUrl:string|null;kind:string};
export type Change={username:string;profileUrl:string|null;category:string;direction:"added"|"removed"};
export type ImportPreview={token:string;sourceName:string;detectedUsername:string|null;followers:number;following:number;warnings:string[]};

export const api={
  accounts:()=>invoke<Account[]>("list_accounts"), snapshots:(accountId:number)=>invoke<Snapshot[]>("list_snapshots",{accountId}),
  summary:(accountId:number,snapshotId?:number)=>invoke<Summary>("get_summary",{accountId,snapshotId}),
  relationships:(snapshotId:number,kind:string,search:string)=>invoke<Relationship[]>("get_relationships",{snapshotId,kind,search}),
  changes:(fromSnapshotId:number,toSnapshotId:number)=>invoke<Change[]>("compare_snapshots",{fromSnapshotId,toSnapshotId}),
  inspect:(path:string)=>invoke<ImportPreview>("inspect_import",{path}),
  commit:(token:string,accountId:number|null,label:string)=>invoke<Snapshot>("commit_import",{token,accountId,label}),
  deleteSnapshot:(snapshotId:number)=>invoke<void>("delete_snapshot",{snapshotId}), deleteAccount:(accountId:number)=>invoke<void>("delete_account",{accountId}),
  exportReport:(snapshotId:number,kind:string,format:string,path:string)=>invoke<void>("export_report",{snapshotId,kind,format,path})
};
