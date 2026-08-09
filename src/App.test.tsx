// @vitest-environment jsdom
import { QueryClient,QueryClientProvider } from "@tanstack/react-query";
import { cleanup,fireEvent,render,screen,waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach,beforeEach,describe,expect,it,vi } from "vitest";
import App from "./App";

const invoke=vi.hoisted(()=>vi.fn());
vi.mock("@tauri-apps/api/core",()=>({invoke}));

const account={id:1,label:"My history",username:"owner",snapshotCount:2};
const newest={id:2,accountId:1,importedAt:"2026-02-01T00:00:00Z",sourceName:"new.zip",followers:2,following:1};
const oldest={id:1,accountId:1,importedAt:"2026-01-01T00:00:00Z",sourceName:"old.zip",followers:1,following:1};

function response(command:string,args?:Record<string,unknown>){
  switch(command){
    case "list_accounts":return [account];
    case "list_snapshots":return [newest,oldest];
    case "get_summary":return {followers:2,following:1,mutuals:1,notFollowingBack:0,followersNotFollowedBack:1,newFollowers:1,lostFollowers:0,hasPreviousSnapshot:true};
    case "get_relationships":return {items:[{username:"alice",profileUrl:null,kind:args?.kind}],nextCursor:null};
    case "compare_snapshots":return {items:[{username:"alice",profileUrl:null,category:"followers",direction:"added"}],nextCursor:null};
    case "get_fame_foundation_status":return {implementationStage:"synthetic_foundation",formulaVersion:"fame-v1",protocolSchemaVersion:1,fixedCorpusRecordBytes:64,networkRetrievalAvailable:false,architectureStatus:"frozen",nextStage:"formal",completedFoundations:["versioned scoring"],blockedGates:["independent PIR operators"]};
    case "rename_account":return {...account,label:args?.label};
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
    await screen.findByText("@alice");
    await userEvent.click(screen.getByRole("button",{name:"Changes"}));
    expect(await screen.findByText("added")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("compare_snapshots",{fromSnapshotId:1,toSnapshotId:2,category:"followers",search:"",after:null,limit:200});
    const historyButtons=screen.getAllByRole("button",{name:/1 follower/});
    await userEvent.click(historyButtons[0]);
    expect(await screen.findByText(/oldest snapshot/i)).toBeTruthy();
  });

  it("shows blocked Fame gates without offering network retrieval",async()=>{
    renderApp();
    await userEvent.click(screen.getByRole("button",{name:"Fame roadmap"}));
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

  it("never falls back to a relationship export from Changes",async()=>{
    renderApp();
    await screen.findByText("@alice");
    await userEvent.click(screen.getByRole("button",{name:"Changes"}));
    await userEvent.click(screen.getAllByRole("button",{name:/1 follower/})[0]);
    await screen.findByText(/oldest snapshot/i);
    await userEvent.click(screen.getByRole("button",{name:"CSV"}));
    expect((await screen.findByRole("alert")).textContent).toMatch(/immediately prior import/i);
    expect(invoke).not.toHaveBeenCalledWith("export_report",expect.anything());
    expect(invoke).not.toHaveBeenCalledWith("export_changes",expect.anything());
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
    await screen.findByText("@alice");
    await userEvent.click(screen.getByRole("button",{name:"Load more"}));
    expect(await screen.findByText("@bob")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("get_relationships",{
      snapshotId:2,kind:"followers",search:"",after:"alice",limit:200
    });
  });

  it("debounces search before querying the native database",async()=>{
    renderApp();
    await screen.findByText("@alice");
    invoke.mockClear();
    await userEvent.type(screen.getByRole("textbox",{name:"Search username"}),"bob");
    await waitFor(()=>expect(invoke).toHaveBeenCalledWith("get_relationships",{
      snapshotId:2,kind:"followers",search:"bob",after:null,limit:200
    }),{timeout:1_000});
  });
});
