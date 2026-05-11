import { test, expect } from "@playwright/test";

test("title", async ({ page }) => {
  let viewport_height = 915;
  // viewport: { width: 412, height: 915 },
  await page.goto("http://localhost:3000");
  await page.waitForTimeout(5000);
  await expect(page.locator('[id="5idoghr47bvsajsi5izx-link"]')).toBeVisible();

  let gallery = page.locator('[id="gallery"]');

  let offset = 1;

  let page_offset_id = await gallery.evaluate(
    (elm) => elm.firstElementChild.id,
  );
  let page_offset_id_str = `[id="${page_offset_id}"]`;
  let page_offset = page.locator(page_offset_id_str);
  let page_offset_y = await page_offset.evaluate(
    (elm) => elm.getBoundingClientRect().y,
  );

  let scroll_iter_index = 1;
  // fisrt fetch
  {
    let elm_count = await gallery.evaluate((elm) => elm.childElementCount);
    expect(elm_count).toBe(22);
  }

  let scroll_down_fn = async () => {
    // let selected_anchors = debug_state.manual_data.filter(v => v.label == "anchor_selected").map(v=> JSON.parse(v.data));
    // let selected_anchors = debug_state.manual_data.filter(v => v.label == "anchor_selected").map(v=> v.data);
    // console.log(`e2e DEBUG STATE2 ${JSON.stringify(selected_anchors)}`);

    // {
    //   let elm_count = await gallery.evaluate((elm) => elm.childElementCount);
    //   expect(elm_count).toBe(22);
    // }

    let debug_state = await page.evaluate(async () =>
      wasm_bindgen.get_debug_state(),
    );
    console.log(`e2e DEBUG STATE ${JSON.stringify(debug_state, null, 2)}`);
    let anchor_last = debug_state.signal_data.filter(
      (v) => v.label == "anchor_last",
    );
    expect(anchor_last.length).toBe(1);
    console.log(`e2e DEBUG STATE1 ${JSON.stringify(anchor_last, null, 2)}`);
    anchor_last = anchor_last[0].value;
    expect(anchor_last.length).toBe(1 + scroll_iter_index);
    anchor_last = JSON.parse(anchor_last[scroll_iter_index]);
    expect(anchor_last).toBeTruthy();
    let anchor_last_id = anchor_last.id;
    console.log(`e2e DEBUG STATE3 ${anchor_last.id}`);
    anchor_last = `[id="${anchor_last.id}"]`;
    // console.log(`e2e DEBUG STATE4 ${anchor_last}`);
    anchor_last = page.locator(anchor_last);
    // let wtf1 = await anchor_last.evaluate((elm) => elm.id);
    // expect(wtf1).toBe("wtf");
    //
    // .map((v) => v.data);

    // let selected_anchors = debug_state.manual_data;
    // console.log(`e2e DEBUG STATE3 ${JSON.stringify(selected_anchors)}`);

    let last_item_id = await gallery.evaluate((elm) => elm.lastElementChild.id);
    let last_item_id_str = `[id="${last_item_id}"]`;
    let last_item = page.locator(last_item_id_str);

    let gallery_height = await gallery.evaluate((elm) => elm.clientHeight);
    let last_item_y = await last_item.evaluate(
      (elm) => elm.getBoundingClientRect().y,
    );

    let scroll_by = last_item_y - (page_offset_y + gallery_height + offset);

    await page.mouse.move(200, 400);

    // SCROLL 1
    await page.mouse.wheel(0, scroll_by);
    await page.waitForTimeout(3000);

    let anchor_last_y_before = await anchor_last.evaluate(
      (elm) => elm.getBoundingClientRect().y,
    );

    console.log(`e2e DEBUG STATE Y BEFORE ${anchor_last_y_before}`);

    last_item_y = await last_item.evaluate(
      (elm) => elm.getBoundingClientRect().top,
    );

    let expected_y = last_item_y + offset;
    let expected_y_left = expected_y - (expected_y % 5);

    // SCROLL 2
    await page.mouse.wheel(0, offset);
    await page.waitForTimeout(3000);

    debug_state = await page.evaluate(async () =>
      wasm_bindgen.get_debug_state(),
    );
    console.log(`e2e DEBUG STATE9 ${JSON.stringify(debug_state, null, 2)}`);

    let anchor_last_y_after = await anchor_last.evaluate(
      (elm) => elm.getBoundingClientRect().y,
    );
    console.log(`e2e DEBUG STATE Y AFTER ${anchor_last_y_after}`);

    {
      let elm_count = await gallery.evaluate((elm) => elm.childElementCount);
      expect(elm_count).toBe(40);
    }

    {
      let gallery_mutated = debug_state.manual_data.filter(
        (v) => v.label == "gallery_mutated",
      );
      expect(gallery_mutated.length).toBe(1 + scroll_iter_index);
    }

    {
      let anchor_selected = debug_state.manual_data.filter(
        (v) => v.label == "anchor_selected",
      );
      expect(anchor_selected.length).toBe(1 + 1 + scroll_iter_index);
      anchor_selected = anchor_selected[anchor_selected.length - 1].data;
      console.log(`e2e DEBUG STATE ANCHOR SELECTED ${anchor_selected}`);
      anchor_selected = JSON.parse(anchor_selected);
    }

    {
      let scroll_offset = debug_state.manual_data.filter(
        (v) => v.label == "scroll_correction",
      );
      // +1 from initial load, +1 from first mutation, +iter_index from scroll
      expect(scroll_offset.length).toBe(1 + 1 + scroll_iter_index);
      scroll_offset = scroll_offset[scroll_offset.length - 1].data;
      console.log(`e2e DEBUG STATE CORRECTION_SCROLL ${anchor_last_y_after}`);
    }

    let expected_anchor_y = anchor_last_y_before + offset;
    let expected_anchor_y_left = expected_anchor_y - (expected_anchor_y % 5);

    let expected_anchor_y_right =
      anchor_last_y_after - (anchor_last_y_after % 5);

    expect(expected_anchor_y_left).toBe(expected_anchor_y_right);

    last_item_y = await last_item.evaluate(
      (elm) => elm.getBoundingClientRect().top,
    );

    let expected_y_right = last_item_y - (last_item_y % 5);

    {
      console.log(`E2E ${expected_y_left} == ${expected_y_right}`);
      expect(expected_y_left).toBe(expected_y_right);
    }

    scroll_iter_index += 1;
  };
  await scroll_down_fn();
  await scroll_down_fn();
  // await scroll_down_fn();
  // second fetch

  // let item2_id = await gallery.evaluate((elm) => elm.firstElementChild.id);
  // let item2_id_str = `[id="${item2_id}"]`;
  // let item3_id = await gallery.evaluate((elm) => elm.lastElementChild.id);
  // let item3_id_str = `[id="${item3_id}"]`;
  // let item2 = page.locator(item2_id_str);
  // let item3 = page.locator(item3_id_str);
  //
  // let item2_top = await item2.evaluate(
  //   (elm) => elm.getBoundingClientRect().top,
  // );
  // let item3_top = await item3.evaluate(
  //   (elm) => elm.getBoundingClientRect().top,
  // );
  //
  // offset = 1;
  // scroll_by = item3_top - (item0_top + gallery_height + offset); // one pixel offset to not load more elements
  // console.log(`E2E scroll by ${scroll_by}`);
  //
  // await page.mouse.wheel(0, scroll_by);
  // await page.waitForTimeout(3000);
  //
  // let item3_top2 = await item3.evaluate(
  //   (elm) => elm.getBoundingClientRect().top,
  // );
  //
  // await page.mouse.wheel(0, offset);
  // await page.waitForTimeout(3000);
  //
  // let item3_top3 = await item3.evaluate(
  //   (elm) => elm.getBoundingClientRect().top,
  // );
  //
  // let debug_state = await page.evaluate(async () =>
  //   wasm_bindgen.get_debug_state(),
  // );
  //
  // let scroll_offset = debug_state.manual_data[0].data;
  // expect(scroll_offset).toBe("10");
  //
  // {
  //   let expected_top = item3_top2 + offset;
  //   let expected_left = expected_top - (expected_top % 5);
  //   let expected_right = item3_top3 - (item3_top3 % 5);
  //   console.log(`E2E ${expected_left} == ${expected_right}`);
  //   expect(expected_left).toBe(expected_right);
  // }

  // await expect(page.locator('[id="1yqvqf06p4h80f7sh9aa-link"]')).toBeVisible();
  //
  //
  // await page.mouse.wheel(0, 2200);
  // await page.waitForTimeout(3000);
  // await page.mouse.wheel(0, 100);
  // await page.waitForTimeout(3000);
  // await expect(page.locator('[id="jvjhhwkm4va9sot7hnkv-link"]')).toBeVisible();

  // Expect a title "to contain" a substring.
  // await expect(page).toHaveTitle(/Playwright/);
});

// test('get started link', async ({ page }) => {
//   await page.goto('https://playwright.dev/');
//
//   // Click the get started link.
//   await page.getByRole('link', { name: 'Get started' }).click();
//
//   // Expects page to have a heading with the name of Installation.
//   await expect(page.getByRole('heading', { name: 'Installation' })).toBeVisible();
// });
