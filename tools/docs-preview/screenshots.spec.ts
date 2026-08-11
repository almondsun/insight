import { expect, test } from "@playwright/test";

test("captures the synthetic Nivune product tour",async({page,baseURL})=>{
  const allowedOrigin=new URL(baseURL!).origin;
  await page.route("**/*",route=>new URL(route.request().url()).origin===allowedOrigin?route.continue():route.abort());
  await page.clock.setFixedTime(new Date("2026-08-11T12:00:00Z"));
  await page.addInitScript(()=>localStorage.clear());
  await page.goto("/tools/docs-preview/index.html");
  await expect(page.getByRole("heading",{name:"Your private audience memory"})).toBeVisible();
  await expect(page).toHaveScreenshot("onboarding.png",{animations:"disabled",caret:"hide"});

  await page.getByRole("button",{name:"Continue"}).click();
  await expect(page.getByRole("heading",{name:"Demo audience"})).toBeVisible();
  await expect(page).toHaveScreenshot("overview.png",{animations:"disabled",caret:"hide"});

  await page.getByRole("button",{name:"Import ZIP"}).click();
  await expect(page.getByText("Ready to import demo-audience-september.zip")).toBeVisible();
  await expect(page).toHaveScreenshot("import-preview.png",{animations:"disabled",caret:"hide"});
  await page.getByRole("button",{name:"Cancel"}).click();

  for(const [name,file] of [["Relationships","relationships.png"],["Changes","changes.png"],["History","history.png"],["Settings","settings.png"]] as const){
    await page.getByRole("button",{name}).click();
    await expect(page).toHaveScreenshot(file,{animations:"disabled",caret:"hide"});
  }
});
