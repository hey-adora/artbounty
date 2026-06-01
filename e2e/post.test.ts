import { test, expect } from "@playwright/test";
import { 
    gallery_search,
    get_parsed_debug_state_fn,
    get_manual_data,
    get_signal_data,
    get_signal_data_latest,
    scroll_down_fn,
} from "./utils";

const MAX_DESCRIPTION_LENGTH = 2000;

test("post_edit_description", async ({ page }) => {
  await page.goto("http://localhost:3000/login");

  await page.locator('[id="email"]').fill("prime1@heyadora.com");
  await page.locator('[id="password"]').fill("prime1@heyadora.com");
  await page.locator('[id="login_btn"]').click();

  // await page.goto("http://localhost:3000");

  // let last_item_id = await gallery.evaluate((elm) => elm.lastElementChild.id);
  await page.locator('[id="gallery"] a').first().click();
      // await page.locator('[id="post_description"]').waitFor();
  // await page.locator('[id="description_cancel_btn"]').waitFor();
  // let word_count = MAX_DESCRIPTION_LENGTH / word.length;

  let word = "hello";
  let iter_index = 0;

  let edit_description_fn = async (word_count) => {
      // let expected_length = word_count * word.length + word_count / 2;
      let text = Array.from(Array(word_count)).map(()=>word).reduce((a, b)=> `${a} ${b}`);
      await page.locator('[id="description_edit_btn"]').click();
      await page.locator('[id="post_description_editable"]').fill(text);
      await page.locator('[id="description_save_btn"]').click();
      await page.locator('[id="post_description"]').waitFor();

      await page.waitForTimeout(1000);

      let parsed_debug2 = await get_parsed_debug_state_fn(page);
      expect(parsed_debug2.post_description_mutation.length).toBe(1 + iter_index);
      expect(parsed_debug2.post_description_mutation[parsed_debug2.post_description_mutation.length - 1])
      .toBe(`${text.length}`);

      // expect(parsed_debug2.post_description_mutation).toBe(1);
      
      let description_length = await page.locator('[id="description_length"]').evaluate((elm) => elm.textContent);
      expect(description_length).toBe(`${text.length}`);
      iter_index += 1;
  };

  let edit_description_err_fn = async (word_count) => {
      let prev_description_length = await page.locator('[id="description_length"]').evaluate((elm) => elm.textContent);
      let text = Array.from(Array(word_count)).map(()=>word).reduce((a, b)=> `${a} ${b}`);
      await page.locator('[id="description_edit_btn"]').click();
      await page.locator('[id="post_description_editable"]').fill(text);
      await page.locator('[id="description_save_btn"]').click();
      await page.locator('[id="description_errors"]').waitFor();

      let parsed_debug2 = await get_parsed_debug_state_fn(page);
      expect(parsed_debug2.post_description_mutation.length).toBe(1 + iter_index);
      expect(parsed_debug2.post_description_mutation[parsed_debug2.post_description_mutation.length - 1])
      .toBe(`${text.length}`);

      let description_length = await page.locator('[id="description_length"]').evaluate((elm) => elm.textContent);
      expect(description_length).toBe(`${text.length}`);

      await page.locator('[id="description_cancel_btn"]').click();
      await page.waitForTimeout(1000);

      description_length = await page.locator('[id="description_length"]').evaluate((elm) => elm.textContent);

      expect(description_length).toBe(prev_description_length);

      iter_index += 1;
  };
  await edit_description_fn(2);
  await edit_description_fn(10);
  // let parsed_debug2 = await get_parsed_debug_state_fn(page);
  // expect(parsed_debug2.post_description_mutation).toBe(1);
  await edit_description_err_fn(MAX_DESCRIPTION_LENGTH / word.length);

  // await page.locator('[id="search"]').fill("one");
  // await page.locator('[id="search"]').focus();
  // await page.keyboard.press("Enter");

  // await page.reload();
  //
  // let first_elm_id_before = await page
  //   .locator('[id="gallery"] > a')
  //   .first()
  //   .evaluate((elm) => elm.id);
  //
  // let value = await page
  //   .locator('[id="search"]')
  //   .first()
  //   .evaluate((elm) => elm.textContent);
  //
  // await page.waitForTimeout(1000);
  //
  // expect(value).toBe("one");
});
