import { describe,expect,it } from "vitest";
import { changeMetric,matchesUsername,validInstagramUsername } from "./presentation";

describe("relationship presentation",()=>{
  it("does not present a baseline snapshot as historical change",()=>{
    expect(changeMetric(false,100,0)).toBe("—");
    expect(changeMetric(true,3,2)).toBe("3 / 2");
  });

  it("matches usernames case-insensitively and trims the query",()=>{
    expect(matchesUsername("Alice.Example"," alice.")).toBe(true);
    expect(matchesUsername("bob","alice")).toBe(false);
  });

  it("validates locally confirmed archive owners",()=>{
    expect(validInstagramUsername("alice.example")).toBe(true);
    expect(validInstagramUsername("=malicious")).toBe(false);
    expect(validInstagramUsername("")).toBe(false);
  });
});
