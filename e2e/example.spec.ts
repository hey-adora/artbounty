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
    let gallery_mutated = debug_state.manual_data.filter(
      (v) => v.label == "gallery_mutated",
    );
    console.log(
      `e2e DEBUG STATE GALLERY_MUTATED ${JSON.stringify(gallery_mutated.length)}`,
    );
    return gallery_mutated;
  };

  let selected_all_fn = (debug_state) => {
    let anchor_selected = debug_state.manual_data.filter(
      (v) => v.label == "anchor_selected",
    );
    console.log(
      `e2e selected_count_all_fn ${JSON.stringify(anchor_selected, null, 2)}`,
    );
    return anchor_selected;
  };

  let selected_fn = (selected_arr) => {
    let o = selected_arr[selected_arr.length - 1];
    console.log(`e2e selected_fn ${JSON.stringify(o, null, 2)}`);
    JSON.parse(o).id;
  };

  let corrected_all_fn = (debug_state) => {
    let scroll_offset = debug_state.manual_data.filter(
      (v) => v.label == "scroll_correction",
    );
    console.log(
      `e2e DEBUG STATE CORRECTION_SCROLL ${JSON.stringify(scroll_offset, null, 2)}`,
    );
    return scroll_offset;
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
    o = `[id="${o.id}"]`;
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

  let scroll_down_fn = async () => {
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
      first_anchor_last_all.length + scroll_iter_index * 2,
    );

    let anchor_last = anchor_last_locator(anchor_last_all);

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

    // SCROLL 2
    await page.mouse.wheel(0, offset);
    await page.waitForTimeout(3000);

    let anchor_y_after = await get_elm_y(anchor_last);

    expect(round_fn(anchor_y_before)).toBe(round_fn(anchor_y_after));

    scroll_iter_index += 1;
  };

  let scroll_up_fn = async () => {

    let first_item_id = await gallery.evaluate((elm) => elm.firstElementChild.id);
    let first_item_id_str = `[id="${first_item_id}"]`;
    let first_item = page.locator(first_item_id_str);

    let item_height = await first_item.evaluate((elm) => elm.clientHeight);
    let first_item_y = await first_item.evaluate(
      (elm) => elm.getBoundingClientRect().y,
    );

    let scroll_by =  first_item_y + ( item_height - (page_offset_y - offset));
    console.log(`e2e scroll_by UP ${scroll_by}`);

    await page.mouse.move(200, 400);

    // SCROLL 1
    await page.mouse.wheel(0, scroll_by);
    await page.waitForTimeout(3000);


    // SCROLL 2
    await page.mouse.wheel(0, -offset);
    await page.waitForTimeout(3000);


    scroll_iter_index += 1;
  };

  await scroll_down_fn();
  await scroll_down_fn();

  await scroll_up_fn();
  await scroll_up_fn();



});
