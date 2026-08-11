// @vitest-environment jsdom
import { QueryClient,QueryClientProvider } from "@tanstack/react-query";
import { cleanup,fireEvent,render,screen,waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach,beforeEach,describe,expect,it,vi } from "vitest";
import App from "./App";

const invoke=vi.hoisted(()=>vi.fn());
vi.mock("@tauri-apps/api/core",()=>({invoke}));

const account={id:1,label:"My history",username:"owner",snapshotCount:2};
const newest={id:2,accountId:1,importedAt:"2026-02-02T00:00:00Z",observedAt:"2026-02-01",observedAtSource:"user_confirmed",sourceName:"new.zip",followers:2,following:1};
const oldest={id:1,accountId:1,importedAt:"2026-01-02T00:00:00Z",observedAt:"2026-01-01",observedAtSource:"legacy_import_time",sourceName:"old.zip",followers:1,following:1};

function response(command:string,args?:Record<string,unknown>){
  switch(command){
    case "list_accounts":return [account];
    case "list_snapshots":return [newest,oldest];
    case "get_summary":return {followers:2,following:1,mutuals:1,notFollowingBack:0,followersNotFollowedBack:1,newFollowers:1,lostFollowers:0,hasPreviousSnapshot:true};
    case "get_trends":return [{snapshotId:1,observedAt:"2026-01-01",followers:1,following:1,mutuals:0,newFollowers:0,lostFollowers:0},{snapshotId:2,observedAt:"2026-02-01",followers:2,following:1,mutuals:1,newFollowers:1,lostFollowers:0}];
    case "get_relationship_history":return [{snapshotId:1,observedAt:"2026-01-01",followsYou:false,youFollow:true},{snapshotId:2,observedAt:"2026-02-01",followsYou:true,youFollow:true}];
    case "get_relationships":return {items:[{username:"alice",profileUrl:null,kind:args?.kind}],nextCursor:null};
    case "compare_snapshots":return {items:[{username:"alice",profileUrl:null,category:"followers",direction:"added"}],nextCursor:null};
    case "get_fame_foundation_status":return {implementationStage:"synthetic_foundation",formulaVersion:"fame-v1",protocolSchemaVersion:1,fixedCorpusRecordBytes:64,networkRetrievalAvailable:false,architectureStatus:"frozen",nextStage:"formal",completedFoundations:["versioned scoring"],blockedGates:["independent PIR operators"]};
    case "rename_account":return {...account,label:args?.label};
    case "create_encrypted_backup":return true;
    case "restore_encrypted_backup":return true;
    default:return undefined;
  }
}

function renderApp(){
  const client=new QueryClient({defaultOptions:{queries:{retry:false,staleTime:Infinity},mutations:{retry:false}}});
  return render(<QueryClientProvider client={client}><App/></QueryClientProvider>);
}

beforeEach(()=>{invoke.mockImplementation((command,args)=>Promise.resolve(response(command,args)))});
afterEach(()=>{cleanup();vi.restoreAllMocks()});

describe("desktop workflow",()=>{
  it("selects a historical snapshot and compares it with the immediately prior import",async()=>{
    renderApp();
    await userEvent.click(await screen.findByRole("button",{name:"Relationships"}));
    await screen.findByText("@alice");
    await userEvent.click(screen.getByRole("button",{name:"Changes"}));
    expect(await screen.findByText("added")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("compare_snapshots",{fromSnapshotId:1,toSnapshotId:2,category:"followers",search:"",after:null,direction:null,limit:200});
    await userEvent.click(screen.getByRole("button",{name:"History"}));
    expect(await screen.findByText(/Date inferred from the old import time/i)).toBeTruthy();
  });

  it("shows blocked Fame gates without offering network retrieval",async()=>{
    renderApp();
    await userEvent.click(screen.getByRole("button",{name:"Settings"}));
    await userEvent.click(screen.getByRole("button",{name:/Open Fame research status/i}));
    expect(await screen.findByText("independent PIR operators")).toBeTruthy();
    expect(screen.queryByRole("button",{name:/start fame/i})).toBeNull();
  });

  it("renames an account through the native command",async()=>{
    vi.spyOn(window,"prompt").mockReturnValue("Renamed");
    renderApp();
    await screen.findByRole("heading",{name:"My history"});
    fireEvent.click(screen.getByRole("button",{name:"Rename"}));
    await waitFor(()=>expect(invoke).toHaveBeenCalledWith("rename_account",{accountId:1,label:"Renamed"}));
  });

  it("exports the explicitly selected comparison from Changes",async()=>{
    renderApp();
    await userEvent.click(screen.getByRole("button",{name:"Changes"}));
    await userEvent.click(screen.getByRole("button",{name:"CSV"}));
    expect(invoke).toHaveBeenCalledWith("export_changes",{fromSnapshotId:1,toSnapshotId:2,category:"followers",direction:null,format:"csv"});
    expect(invoke).not.toHaveBeenCalledWith("export_report",expect.anything());
  });

  it("confirms destructive account deletion before invoking native code",async()=>{
    vi.spyOn(window,"confirm").mockReturnValue(true);
    renderApp();
    await screen.findByRole("heading",{name:"My history"});
    await userEvent.click(screen.getByRole("button",{name:"Delete"}));
    await waitFor(()=>expect(invoke).toHaveBeenCalledWith("delete_account",{accountId:1}));
  });

  it("loads the next relationship page from the native cursor",async()=>{
    invoke.mockImplementation((command,args)=>{
      if(command==="get_relationships")return Promise.resolve(args?.after
        ?{items:[{username:"bob",profileUrl:null,kind:"followers"}],nextCursor:null}
        :{items:[{username:"alice",profileUrl:null,kind:"followers"}],nextCursor:"alice"});
      return Promise.resolve(response(command,args));
    });
    renderApp();
    await userEvent.click(await screen.findByRole("button",{name:"Relationships"}));
    await screen.findByText("@alice");
    await userEvent.click(screen.getByRole("button",{name:"Load more"}));
    expect(await screen.findByText("@bob")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("get_relationships",{
      snapshotId:2,kind:"followers",search:"",after:"alice",limit:200
    });
  });

  it("debounces search before querying the native database",async()=>{
    renderApp();
    await userEvent.click(await screen.findByRole("button",{name:"Relationships"}));
    await screen.findByText("@alice");
    invoke.mockClear();
    fireEvent.change(screen.getByRole("textbox",{name:"Search username"}),{target:{value:"bob"}});
    await waitFor(()=>expect(invoke).toHaveBeenCalledWith("get_relationships",{
      snapshotId:2,kind:"followers",search:"bob",after:null,limit:200
    }),{timeout:1_000});
  });

  it("applies smart-list direction in the native query",async()=>{
    renderApp();
    await userEvent.click(screen.getByRole("button",{name:"Changes"}));
    await userEvent.click(screen.getByRole("button",{name:"Lost followers"}));
    await waitFor(()=>expect(invoke).toHaveBeenCalledWith("compare_snapshots",{
      fromSnapshotId:1,toSnapshotId:2,category:"followers",search:"",after:null,direction:"removed",limit:200
    }));
  });

  it("creates an encrypted backup only after a sufficiently long passphrase",async()=>{
    renderApp();
    await userEvent.click(screen.getByRole("button",{name:"Settings"}));
    const create=screen.getByRole("button",{name:"Create backup"});
    expect(create).toHaveProperty("disabled",true);
    fireEvent.change(screen.getByLabelText("Backup passphrase"),{target:{value:"a secure backup phrase"}});
    fireEvent.change(screen.getByLabelText("Confirm backup passphrase"),{target:{value:"a secure backup phrase"}});
    await userEvent.click(create);
    await waitFor(()=>expect(invoke).toHaveBeenCalledWith("create_encrypted_backup",{passphrase:"a secure backup phrase"}));
  });
});
