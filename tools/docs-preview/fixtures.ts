import type { NivuneApi, Snapshot } from "../../src/api";

const snapshots: Snapshot[] = [
  {id:4,accountId:1,importedAt:"2026-08-02T14:00:00Z",observedAt:"2026-08-01",observedAtSource:"user_confirmed",sourceName:"demo-audience-august.zip",followers:1842,following:611},
  {id:3,accountId:1,importedAt:"2026-07-03T14:00:00Z",observedAt:"2026-07-01",observedAtSource:"user_confirmed",sourceName:"demo-audience-july.zip",followers:1764,following:604},
  {id:2,accountId:1,importedAt:"2026-06-03T14:00:00Z",observedAt:"2026-06-01",observedAtSource:"user_confirmed",sourceName:"demo-audience-june.zip",followers:1698,following:597},
  {id:1,accountId:1,importedAt:"2026-05-03T14:00:00Z",observedAt:"2026-05-01",observedAtSource:"legacy_import_time",sourceName:"demo-audience-may.zip",followers:1631,following:590},
];

const people=["amber_field","cedar_notes","cobalt_studio","green_orbit","harbor_lines","lumen_garden","moss_archive","paper_sky"];

export const docsApi: NivuneApi = {
  accounts:async()=>[{id:1,label:"Demo audience",username:"demo_owner",snapshotCount:4}],
  snapshots:async()=>snapshots,
  summary:async()=>({followers:1842,following:611,mutuals:487,notFollowingBack:124,followersNotFollowedBack:1355,newFollowers:96,lostFollowers:18,hasPreviousSnapshot:true}),
  trends:async()=>[
    {snapshotId:1,observedAt:"2026-05-01",followers:1631,following:590,mutuals:441,newFollowers:0,lostFollowers:0},
    {snapshotId:2,observedAt:"2026-06-01",followers:1698,following:597,mutuals:455,newFollowers:82,lostFollowers:15},
    {snapshotId:3,observedAt:"2026-07-01",followers:1764,following:604,mutuals:469,newFollowers:89,lostFollowers:23},
    {snapshotId:4,observedAt:"2026-08-01",followers:1842,following:611,mutuals:487,newFollowers:96,lostFollowers:18},
  ],
  relationshipHistory:async(_accountId,username)=>snapshots.slice().reverse().map((snapshot,index)=>({snapshotId:snapshot.id,observedAt:snapshot.observedAt,followsYou:index>0,youFollow:username!=="paper_sky"})),
  relationships:async(_snapshotId,kind,search)=>({items:people.filter(name=>name.includes(search)).map(username=>({username,profileUrl:null,kind})),nextCursor:null}),
  changes:async(_fromId,_toId,category,search,_after,direction)=>({items:people.filter(name=>name.includes(search)).slice(0,6).map((username,index)=>({username,profileUrl:null,category,direction:(direction??(index<4?"added":"removed")) as "added"|"removed"})),nextCursor:null}),
  renameAccount:async(_accountId,label)=>({id:1,label,username:"demo_owner",snapshotCount:4}),
  fameFoundationStatus:async()=>({implementationStage:"synthetic_foundation",formulaVersion:"fame-v1",protocolSchemaVersion:1,fixedCorpusRecordBytes:64,networkRetrievalAvailable:false,architectureStatus:"frozen",nextStage:"formal threat model",completedFoundations:["Versioned scoring","Synthetic corpus tooling"],blockedGates:["Independent PIR operators","Audited mixnet deployment"]}),
  chooseImport:async()=>({token:"synthetic-preview",sourceName:"demo-audience-september.zip",detectedUsername:"demo_owner",followers:1914,following:618,warnings:["This synthetic documentation archive has no profile links."]}),
  commit:async()=>snapshots[0],
  cancelImport:async()=>undefined,
  deleteSnapshot:async()=>undefined,
  deleteAccount:async()=>undefined,
  exportReport:async()=>true,
  exportChanges:async()=>true,
  createEncryptedBackup:async()=>true,
  restoreEncryptedBackup:async()=>true,
};
