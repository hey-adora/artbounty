import { test, expect } from "@playwright/test";

test("title", async ({ page }) => {
  let viewport_height = 915;
  // viewport: { width: 412, height: 915 },
  await page.goto("http://localhost:3000");
  await page.waitForTimeout(5000);
  await expect(page.locator('[id="5idoghr47bvsajsi5izx-link"]')).toBeVisible();

  let gallery = page.locator('[id="gallery"]');

  let offset = 1;
  let scroll_iter_index = 0;

  let page_offset_id = await gallery.evaluate(
    (elm) => elm.firstElementChild.id,
  );
  let page_offset_id_str = `[id="${page_offset_id}"]`;
  let page_offset = page.locator(page_offset_id_str);
  let page_offset_y = await page_offset.evaluate(
    (elm) => elm.getBoundingClientRect().y,
  );

  // fisrt fetch
  {
    let elm_count = await gallery.evaluate((elm) => elm.childElementCount);
    expect(elm_count).toBe(22);
  }

  let get_debug_state_fn = async () => {
    let debug_state = await page.evaluate(async () =>
      wasm_bindgen.get_debug_state(),
    );
    return debug_state;
  };

  let mutated_all_fn = (debug_state) => {
    // let debug_state = await page.evaluate(async () =>
    //   wasm_bindgen.get_debug_state(),
    // );
    let gallery_mutated = debug_state.manual_data.filter(
      (v) => v.label == "gallery_mutated",
    );
    console.log(
      `e2e DEBUG STATE GALLERY_MUTATED ${JSON.stringify(gallery_mutated.length)}`,
    );
    // +1 init, +1 first mutation, +1 ??
    // expect(gallery_mutated.length).toBe(count);
    return gallery_mutated;
  };

  let selected_all_fn = (debug_state) => {
    let anchor_selected = debug_state.manual_data.filter(
      (v) => v.label == "anchor_selected",
    );
    // expect(anchor_selected.length).toBe(1 + 1 + 1 + scroll_iter_index * 1);
    // anchor_selected = anchor_selected[anchor_selected.length - 1].data;
    console.log(
      `e2e selected_count_all_fn ${JSON.stringify(anchor_selected, null, 2)}`,
    );
    // anchor_selected = JSON.parse(anchor_selected);
    return anchor_selected;
  };

  let selected_fn = (selected_arr) => {
    let o = selected_arr[selected_arr.length - 1];
    console.log(`e2e selected_fn ${JSON.stringify(o, null, 2)}`);
    JSON.parse(o).id;
  };

  let corrected_all_fn = (debug_state) => {
    // let debug_state = await page.evaluate(async () =>
    //   wasm_bindgen.get_debug_state(),
    // );
    let scroll_offset = debug_state.manual_data.filter(
      (v) => v.label == "scroll_correction",
    );
    console.log(
      `e2e DEBUG STATE CORRECTION_SCROLL ${JSON.stringify(scroll_offset, null, 2)}`,
    );
    return scroll_offset;
    // +1 from initial load, +1 from first mutation, +iter_index from scroll
    // expect(scroll_offset.length).toBe(count);
    // scroll_offset = scroll_offset[scroll_offset.length - 1].data;
    // console.log(`e2e DEBUG STATE CORRECTION_SCROLL ${anchor_last_y_after}`);
  };
  let corrected_succesfull_fn = (corrected_count) => {
    let o = corrected_count.filter((v) => v.data != "null");
    console.log(
      `e2e corrected_succesfull_count_fn ${JSON.stringify(o, null, 2)}`,
    );
    return o;
  };

  let anchor_last_all_fn = (debug_state) => {
    console.log(`e2e DEBUG STATE ${JSON.stringify(debug_state, null, 2)}`);
    let anchor_last = debug_state.signal_data.filter(
      (v) => v.label == "anchor_last",
    );
    expect(anchor_last.length).toBe(1);
    console.log(`e2e DEBUG STATE1 ${JSON.stringify(anchor_last, null, 2)}`);
    anchor_last = anchor_last[0].data;
    return anchor_last;
    // console.log(`e2e DEBUG STATE1.2 ${JSON.stringify(anchor_last, null, 2)}`);
    // // +1 for init, +1 for first request, +0 for first mutation
    // expect(anchor_last.length).toBe(2 + scroll_iter_index * 2);
    // anchor_last = JSON.parse(anchor_last[anchor_last.length - 1]);
    // expect(anchor_last).toBeTruthy();
    // let anchor_last_id = anchor_last.id;
    // console.log(`e2e DEBUG STATE3 ${anchor_last.id}`);
    // anchor_last = `[id="${anchor_last.id}"]`;
    // // console.log(`e2e DEBUG STATE4 ${anchor_last}`);
    // anchor_last = page.locator(anchor_last);
  };

  let anchor_last_successfull_fn = (v) => {
    let o = v.filter((v) => v != "null");
    console.log(`e2e anchor_last_successfull_fn ${JSON.stringify(o, null, 2)}`);
    return o;
  };

  let anchor_last_locator = (anchor_last_arr) => {
    let o = anchor_last_arr[anchor_last_arr.length - 1];
    o = JSON.parse(o);
    console.log(`e2e anchor_last_locator ${JSON.stringify(o, null, 2)}`);
    // let anchor_last_id = anchor_last.id;
    // // console.log(`e2e DEBUG STATE3 ${anchor_last.id}`);
    o = `[id="${o.id}"]`;
    // // console.log(`e2e DEBUG STATE4 ${anchor_last}`);
    o = page.locator(o);

    return o;
  };

  let get_elm_y = async (elm_locator) => {
    let y = await elm_locator.evaluate((elm) => elm.getBoundingClientRect().y);
    console.log(`e2e get_elm_y ${y}`);
    return y;
  };

  let round_fn = (num) => {
      return num - (num % 5);
  };

  let first_debug_state = await get_debug_state_fn();

  let first_mutated_all = mutated_all_fn(first_debug_state);

  let first_selected_all = selected_all_fn(first_debug_state);

  let first_corrected_all = corrected_all_fn(first_debug_state);
  let first_corrected_successfull =
    corrected_succesfull_fn(first_corrected_all);

  let first_anchor_last_all = anchor_last_all_fn(first_debug_state);
  let first_anchor_last_successfull = anchor_last_successfull_fn(
    first_anchor_last_all,
  );
  let first_anchor_last = anchor_last_locator(first_anchor_last_successfull);

  expect(first_anchor_last).toBeTruthy();

  // anchor_last_fn(debug_state);

  // expect(anchor_last_successfull.length).toBe(1);
  // expect(corrected_successfull.length).toBe(10);

  let scroll_down_fn = async () => {
    // let selected_anchors = debug_state.manual_data.filter(v => v.label == "anchor_selected").map(v=> JSON.parse(v.data));
    // let selected_anchors = debug_state.manual_data.filter(v => v.label == "anchor_selected").map(v=> v.data);
    // console.log(`e2e DEBUG STATE2 ${JSON.stringify(selected_anchors)}`);

    // {
    //   let elm_count = await gallery.evaluate((elm) => elm.childElementCount);
    //   expect(elm_count).toBe(22);
    // }

    let debug_state = await get_debug_state_fn();

    let mutated_all = mutated_all_fn(debug_state);
    let selected_all = selected_all_fn(debug_state);
    let corrected_all = corrected_all_fn(debug_state);
    let anchor_last_all = anchor_last_all_fn(debug_state);

    expect(mutated_all.length).toBe(
      first_mutated_all.length + scroll_iter_index,
    );

    expect(selected_all.length).toBe(
      first_selected_all.length + scroll_iter_index,
    );

    expect(corrected_all.length).toBe(
      first_corrected_all.length + scroll_iter_index,
    );

    expect(anchor_last_all.length).toBe(
      first_anchor_last_all.length + (scroll_iter_index * 2),
    );

    let anchor_last = anchor_last_locator(anchor_last_all);

    // expect(1).toBe(2);

    // let debug_state = await page.evaluate(async () =>
    //   wasm_bindgen.get_debug_state(),
    // );
    // console.log(`e2e DEBUG STATE ${JSON.stringify(debug_state, null, 2)}`);
    // let anchor_last = debug_state.signal_data.filter(
    //   (v) => v.label == "anchor_last",
    // );
    // expect(anchor_last.length).toBe(1);
    // console.log(`e2e DEBUG STATE1 ${JSON.stringify(anchor_last, null, 2)}`);
    // anchor_last = anchor_last[0].data;
    // console.log(`e2e DEBUG STATE1.2 ${JSON.stringify(anchor_last, null, 2)}`);
    // // +1 for init, +1 for first request, +0 for first mutation
    // expect(anchor_last.length).toBe(2 + scroll_iter_index * 2);
    // anchor_last = JSON.parse(anchor_last[anchor_last.length - 1]);
    // expect(anchor_last).toBeTruthy();
    // let anchor_last_id = anchor_last.id;
    // console.log(`e2e DEBUG STATE3 ${anchor_last.id}`);
    // anchor_last = `[id="${anchor_last.id}"]`;
    // // console.log(`e2e DEBUG STATE4 ${anchor_last}`);
    // anchor_last = page.locator(anchor_last);
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

    let anchor_y_before = await get_elm_y(anchor_last);
    // let anchor_last_y_before = await anchor_last.evaluate(
    //   (elm) => elm.getBoundingClientRect().y,
    // );

    // console.log(`e2e DEBUG STATE Y BEFORE ${anchor_last_y_before}`);

    // last_item_y = await last_item.evaluate(
    //   (elm) => elm.getBoundingClientRect().top,
    // );

    // let expected_y = last_item_y + offset;
    // let expected_y_left = expected_y - (expected_y % 5);

    // SCROLL 2
    await page.mouse.wheel(0, offset);
    await page.waitForTimeout(3000);

    let anchor_y_after = await get_elm_y(anchor_last);

    expect(round_fn(anchor_y_before)).toBe(round_fn(anchor_y_after));

    // first_debug_state = await page.evaluate(async () =>
    //   wasm_bindgen.get_debug_state(),
    // );
    // console.log(
    //   `e2e DEBUG STATE9 ${JSON.stringify(first_debug_state, null, 2)}`,
    // );
    //
    // let anchor_last_y_after = await anchor_last.evaluate(
    //   (elm) => elm.getBoundingClientRect().y,
    // );
    // console.log(`e2e DEBUG STATE Y AFTER ${anchor_last_y_after}`);

    // {
    //   let elm_count = await gallery.evaluate((elm) => elm.childElementCount);
    //   expect(elm_count).toBe(40);
    // }

    // {
    //   first_debug_state = await page.evaluate(async () =>
    //     wasm_bindgen.get_debug_state(),
    //   );
    //   let gallery_mutated = first_debug_state.manual_data.filter(
    //     (v) => v.label == "gallery_mutated",
    //   );
    //   console.log(
    //     `e2e DEBUG STATE GALLERY_MUTATED ${JSON.stringify(gallery_mutated.length)}`,
    //   );
    //   // +1 init, +1 first mutation, +1 ??
    //   expect(gallery_mutated.length).toBe(2 + scroll_iter_index);
    // }
    //
    // {
    //   let anchor_selected = first_debug_state.manual_data.filter(
    //     (v) => v.label == "anchor_selected",
    //   );
    //   expect(anchor_selected.length).toBe(1 + 1 + 1 + scroll_iter_index * 1);
    //   anchor_selected = anchor_selected[anchor_selected.length - 1].data;
    //   console.log(`e2e DEBUG STATE ANCHOR SELECTED ${anchor_selected}`);
    //   anchor_selected = JSON.parse(anchor_selected);
    // }

    // await scroll_correction_count_assert_fn(1);

    // {
    //   let scroll_offset = debug_state.manual_data.filter(
    //     (v) => v.label == "scroll_correction",
    //   );
    //   // +1 from initial load, +1 from first mutation, +iter_index from scroll
    //   expect(scroll_offset.length).toBe(1 + 1 + scroll_iter_index * 2);
    //   scroll_offset = scroll_offset[scroll_offset.length - 1].data;
    //   console.log(`e2e DEBUG STATE CORRECTION_SCROLL ${anchor_last_y_after}`);
    // }

    // let expected_anchor_y = anchor_last_y_before + offset;
    // let expected_anchor_y_left = expected_anchor_y - (expected_anchor_y % 5);
    //
    // let expected_anchor_y_right =
    //   anchor_last_y_after - (anchor_last_y_after % 5);
    //
    // expect(expected_anchor_y_left).toBe(expected_anchor_y_right);
    //
    // last_item_y = await last_item.evaluate(
    //   (elm) => elm.getBoundingClientRect().top,
    // );
    //
    // let expected_y_right = last_item_y - (last_item_y % 5);
    //
    // {
    //   console.log(`E2E ${expected_y_left} == ${expected_y_right}`);
    //   expect(expected_y_left).toBe(expected_y_right);
    // }

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
