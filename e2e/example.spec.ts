import { test, expect } from "@playwright/test";

test("infinite_scroll", async ({ page }) => {
  let viewport_height = 915;
  // viewport: { width: 412, height: 915 },
  await page.goto("http://localhost:3000");

  await page.locator('[id="gallery"] > a').first().waitFor();

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
    let anchor_y_before = await get_elm_y(anchor_last);
    // await page.screenshot({ path: `${scroll_iter_index}_down_0.jpg` });

    // SCROLL 2
    await page.mouse.wheel(0, offset);

    await page.locator(`[id="${last_item_id}"] + a`).waitFor();
    // await page.screenshot({ path: `${scroll_iter_index}_down_1.jpg` });

    let anchor_y_after = await get_elm_y(anchor_last);

    expect(round_fn(anchor_y_before)).toBe(round_fn(anchor_y_after));

    scroll_iter_index += 1;
  };

  let scroll_up_fn = async () => {
    let first_item_id = await gallery.evaluate(
      (elm) => elm.firstElementChild.id,
    );
    let first_item_id_str = `[id="${first_item_id}"]`;
    let first_item = page.locator(first_item_id_str);

    let item_height = await first_item.evaluate((elm) => elm.clientHeight);
    let first_item_y = await first_item.evaluate(
      (elm) => elm.getBoundingClientRect().y,
    );

    let scroll_by = first_item_y + (item_height - (page_offset_y - offset));
    console.log(`e2e scroll_by UP ${scroll_by}`);

    await page.mouse.move(200, 400);

    // SCROLL 1
    await page.mouse.wheel(0, scroll_by);
    await page.locator(first_item_id_str).first().waitFor();
    // await page.screenshot({ path: `${scroll_iter_index}_up_0.jpg` });

    // SCROLL 2
    await page.mouse.wheel(0, -offset);
    await page.locator(`[id="${first_item_id}"] + a`).waitFor();
    // await page.screenshot({ path: `${scroll_iter_index}_up_1.jpg` });

    scroll_iter_index += 1;
  };

  await scroll_down_fn();
  await scroll_down_fn();

  await scroll_up_fn();
  await scroll_up_fn();
});

test("scroll_save_position", async ({ page }) => {
  await page.goto("http://localhost:3000");

  let first_elm_id_before = await page
    .locator('[id="gallery"] > a')
    .first()
    .evaluate((elm) => elm.id);
  let page_offset_y = await page
    .locator('[id="gallery"] > a')
    .first()
    .evaluate((elm) => elm.getBoundingClientRect().y);
  let gallery = page.locator('[id="gallery"]');
  let offset = 1;
  let scroll_iter_index = 0;

  let get_debug_state_fn = async () => {
    let debug_state = await page.evaluate(async () =>
      wasm_bindgen.get_debug_state(),
    );
    console.log(`e2e DEBUG STATE ${JSON.stringify(debug_state, null, 2)}`);
    return debug_state;
  };

  let get_gallery_items = (debug_state) => {
    let output = debug_state.signal_data.filter(
      (v) => v.label == "gallery_api_items",
    );
    expect(output.length).toBe(1);
    output = output[0].data.map((v) => JSON.parse(v));
    output = output[output.length - 1];
    console.log(`e2e GALLERY ITEMS ${JSON.stringify(output, null, 2)}`);
    return output;
  };

  let scroll_down_fn = async () => {
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
    await page.locator(`[id="${last_item_id}"]`).waitFor();

    // SCROLL 2
    await page.mouse.wheel(0, offset);
    await page.locator(`[id="${last_item_id}"] + a`).waitFor();

    scroll_iter_index += 1;
  };

  // {
  //   let debug = await get_debug_state_fn();
  //   let gallery_items = get_gallery_items(debug);
  //   let time = gallery_items[0].created_at;
  //   expect(time).toBe("1777452967556484570");
  //   // 1777452967556484570
  // }

  await scroll_down_fn();
  // {
  //   let debug = await get_debug_state_fn();
  //   let gallery_items = get_gallery_items(debug);
  //   let time = gallery_items[0].created_at;
  //   expect(time).toBe("1777452967556484570");
  //   // 1777452967556484570
  // }
  // await page.mouse.move(200, 400);
  // await page.mouse.wheel(0, 100);
  await page.waitForTimeout(1000);

  // {
  //   let debug = await get_debug_state_fn();
  //   let gallery_items = get_gallery_items(debug);
  //   let time = gallery_items[0].created_at;
  //   expect(time).toBe("1777452967556484570");
  //   // 1777452967556484570
  //   // 1777452967556484570
  // }

  //
  // top_before = await gallery.evaluate((elm) => elm.scrollTop);
  // let params = page.url();
  let top_before = await gallery.evaluate((elm) => elm.scrollTop);
  let params = await page.evaluate(() => {
    let params = new URLSearchParams(document.location.search);
    let direction = params.get("direction");
    let time = params.get("time");
    let scroll = params.get("scroll");
    // return `scroll=${scroll}`;
    return `direction=${direction}&time=${time}&scroll=${scroll}`;
  });

  let debug = await get_debug_state_fn();
  let gallery_items = get_gallery_items(debug);
  let time = gallery_items[0].created_at;
  expect(time).toBe("1777452967556484570");
  // 1777452967556484570
  expect(params).toBe(
    // `scroll=${top_before}`,
    `direction=down&time=${time}&scroll=${top_before}`,
  );

  // await expect(page).toHaveURL((url) => {
  //   const params = url.searchParams;
  //   return (
  //     params.get("direction") === "UP"
  //     // params.has("search") && params.has("options") && params.get("id") === "5"
  //   );
  // });

  // expect(JSON.stringify(params, null, 2)).toBe("q");
  await page.reload();
  let first_elm_id_after = await page
    .locator('[id="gallery"] > a')
    .first()
    .evaluate((elm) => elm.id);

  // await page.waitForTimeout(1000);

  // expect(first_elm_id_before).toBe(first_elm_id_after);

  let top_after = await gallery.evaluate((elm) => elm.scrollTop);
  expect(top_before).toBe(top_after);
});
