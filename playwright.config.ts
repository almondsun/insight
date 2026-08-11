import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir:"./tools/docs-preview",
  testMatch:"screenshots.spec.ts",
  workers:1,
  fullyParallel:false,
  retries:0,
  reporter:"line",
  expect:{toHaveScreenshot:{maxDiffPixelRatio:0.005,threshold:0.2}},
  snapshotPathTemplate:"{testDir}/../../docs/assets/screenshots/{arg}{ext}",
  use:{
    baseURL:"http://127.0.0.1:4173",
    browserName:"chromium",
    viewport:{width:1440,height:900},
    locale:"en-US",
    timezoneId:"UTC",
    colorScheme:"light",
    deviceScaleFactor:1,
  },
  webServer:{command:"npm run docs:preview",url:"http://127.0.0.1:4173/tools/docs-preview/index.html",reuseExistingServer:false,timeout:120_000},
});
