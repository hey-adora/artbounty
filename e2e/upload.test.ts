
import { test, expect } from "@playwright/test";
import { 
    gallery_search,
    get_parsed_debug_state_fn,
    get_manual_data,
    get_signal_data,
    get_signal_data_latest,
    scroll_down_fn,
    login,
} from "./utils";
import * as path from 'path';

test("upload_img", async ({ page }) => {
  await login(page);
  await page.locator('[id="gallery"] > a').first().waitFor();
  await page.goto("http://localhost:3000/upload");
    // await page.waitForTimeout(1000);
  const fileChooserPromise = page.waitForEvent('filechooser');
  await page.locator('[id="image"]').click();
  const fileChooser = await fileChooserPromise;
  // expect(__dirname).toBe(0)
  await fileChooser.setFiles(path.join(__dirname, '../flake.nix'));
  await page.locator('[id="title"]').fill("one");
  // await page.locator('[id="upload_btn"]').click();

  await page.waitForTimeout(5000);
});
