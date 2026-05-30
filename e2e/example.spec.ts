import { test, expect } from "@playwright/test";

test("infinite_scroll", async ({ page }) => {
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

  let get_elm_y = async (elm_locator) => {
    let y = await elm_locator.evaluate((elm) => elm.getBoundingClientRect().y);
    console.log(`e2e get_elm_y ${y}`);
    return y;
  };

  let round_fn = (num) => {
    return num - (num % 5);
  };

  let parsed_debug1 = await get_parsed_debug_state_fn(page);
  let first_anchor_last = await page.locator(`[id="${get_signal_data_latest(parsed_debug1.anchor_last).id}"]`);
  expect(first_anchor_last).toBeTruthy();

  let scroll_down_fn = async () => {
    let parsed_debug2 = await get_parsed_debug_state_fn(page);

    expect(parsed_debug2.count_mutated).toBe(
      parsed_debug1.count_mutated + scroll_iter_index,
    );

    expect(parsed_debug2.count_anchor_selected).toBe(
      parsed_debug1.count_anchor_selected + scroll_iter_index,
    );

    expect(parsed_debug2.count_scroll_corrected).toBe(
      parsed_debug1.count_scroll_corrected + scroll_iter_index,
    );

    expect(parsed_debug2.anchor_last[parsed_debug2.anchor_last.length - 1].length).toBe(
      parsed_debug1.anchor_last[parsed_debug1.anchor_last.length - 1].length + scroll_iter_index * 2,
    );

    let anchor_last = await page.locator(`[id="${get_signal_data_latest(parsed_debug2.anchor_last).id}"]`);

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
    let parsed_debug2 = await get_parsed_debug_state_fn(page);
    let anchor = page.locator(`[id="${get_signal_data_latest(parsed_debug2.anchor_first).id}"]`);

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
    let anchor_y_before = await get_elm_y(anchor);
    // await page.screenshot({ path: `${scroll_iter_index}_up_0.jpg` });

    // SCROLL 2
    await page.mouse.wheel(0, -offset);
    await page.waitForTimeout(1000);
    // await page.locator(`[id="${first_item_id}"] - a`).waitFor();
    let anchor_y_after = await get_elm_y(anchor);
    // await page.screenshot({ path: `${scroll_iter_index}_up_1.jpg` });

    expect(round_fn(anchor_y_before)).toBe(round_fn(anchor_y_after));

    scroll_iter_index += 1;
  };

  await scroll_down_fn();
  await scroll_down_fn();

  await scroll_up_fn();
  await scroll_up_fn();

  let parsed_debug3 = await get_parsed_debug_state_fn(page);
  expect(parsed_debug3.count_scroll_correction_reset).toBe(0);
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

  // let parsed_debug1 = await get_parsed_debug_state_fn(page);

  await scroll_down_fn();

  let top_before = await gallery.evaluate((elm) => elm.scrollTop);
  let params = await page.evaluate(() => {
    let params = new URLSearchParams(document.location.search);
    let direction = params.get("direction");
    let time = params.get("time");
    let scroll = params.get("scroll");
    return `direction=${direction}&time=${time}&scroll=${scroll}`;
  });

  let parsed_debug2 = await get_parsed_debug_state_fn(page);
  let gallery_items = get_signal_data_latest(parsed_debug2.gallery_items);

  let time = gallery_items[0].created_at;
  expect(params).toBe(
    `direction=down&time=${time}&scroll=${top_before}`,
  );

  await page.reload();
  let first_elm_id_after = await page
    .locator('[id="gallery"] > a')
    .first()
    .evaluate((elm) => elm.id);

  let top_after = await gallery.evaluate((elm) => elm.scrollTop);
  // TODO compare items too perhaps, should be same ones
  expect(top_before).toBe(top_after);

  let parsed_debug3 = await get_parsed_debug_state_fn(page);
  expect(parsed_debug3.count_scroll_correction_reset).toBe(0);
});

test("reset_query", async ({ page }) => {
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
  let parsed_debug = await get_parsed_debug_state_fn(page);
  let gallery_items = get_signal_data_latest(parsed_debug.gallery_items);
  let first_item_time = gallery_items[0].created_at;
  await scroll_down_fn(page, gallery, offset, page_offset_y, scroll_iter_index);
  await page.locator('[id="gallery"] > a').first().waitFor();

  let banner = page.locator('[id="banner"]');
  await banner.click();
  await page.waitForTimeout(1000);
  let first_elm_id_after = await page
    .locator('[id="gallery"] > a')
    .first()
    .evaluate((elm) => elm.id);

  expect(first_elm_id_before).toBe(first_elm_id_after);

  let params2 = await page.evaluate(() => {
    let params = new URLSearchParams(document.location.search);
    let direction = params.get("direction");
    let time = params.get("time");
    let scroll = params.get("scroll");
    return `direction=${direction}&time=${time}&scroll=${scroll}`;
  });

  expect(params2).toBe(`direction=down&time=${first_item_time}&scroll=null`);
  let parsed_debug2 = await get_parsed_debug_state_fn(page);
  expect(parsed_debug2.count_reset).toBe(1);
});

test("gallery_search2", async ({ page }) => {
  await page.goto("http://localhost:3000");

  let first_elm_id_before = await page
    .locator('[id="gallery"] > a')
    .first()
    .evaluate((elm) => elm.id);

  let first_debug = await get_parsed_debug_state_fn(page);
  // expect(param_limit).toBe(3);

  await gallery_search(page, first_debug, 1, 1, "dragon", "null");
  // mutation index gets +0 because we run gallery.reset() BUT gallery was already empty
  await gallery_search(page, first_debug, 2, 2, "", "22");
  // mutation index gets +1 because we run gallery.reset()
  await gallery_search(page, first_debug, 3, 4, "one", "3");
  await gallery_search(page, first_debug, 4, 6, "two", "2");
  await gallery_search(page, first_debug, 5, 8, "three", "1");
  await gallery_search(page, first_debug, 6, 10, "one", "3");
  await gallery_search(page, first_debug, 7, 12, "three", "1");
  await gallery_search(page, first_debug, 8, 14, "", "22");
  await gallery_search(page, first_debug, 9, 16, "ONE", "3");
});

test("gallery_search_from_diffrent_page", async ({ page }) => {
  await page.goto("http://localhost:3000/login");

  await page.locator('[id="search"]').fill("");
  await page.locator('[id="search"]').focus();
  await page.keyboard.press("Enter");

  let first_elm_id_before = await page
    .locator('[id="gallery"] > a')
    .first()
    .evaluate((elm) => elm.id);
});

test("gallery_search_input_text_from_url", async ({ page }) => {
  await page.goto("http://localhost:3000");

  await page.locator('[id="search"]').fill("one");
  await page.locator('[id="search"]').focus();
  await page.keyboard.press("Enter");

  await page.reload();

  let first_elm_id_before = await page
    .locator('[id="gallery"] > a')
    .first()
    .evaluate((elm) => elm.id);

  let value = await page
    .locator('[id="search"]')
    .first()
    .evaluate((elm) => elm.textContent);

  await page.waitForTimeout(1000);

  expect(value).toBe("one");
});

let gallery_search = async (
  page,
  first_parsed_debug,
  index,
  mut_index,
  text,
  img_count,
) => {
  await page.locator('[id="search"]').fill(text);
  await page.locator('[id="search"]').focus();
  await page.keyboard.press("Enter");
  await page.waitForTimeout(1000);

  let new_debug = await get_parsed_debug_state_fn(page);

  expect(
    `init_executed ${new_debug.count_init}
reset_executed ${new_debug.count_reset}
param_limit ${new_debug.count_gallery_param_limit}
mutated ${new_debug.count_mutated}
interval_top ${new_debug.count_interval_top}
interval_down ${new_debug.count_interval_down}`,
  ).toBe(
    `init_executed ${first_parsed_debug.count_init}
reset_executed ${first_parsed_debug.count_reset + index}
param_limit ${first_parsed_debug.count_gallery_param_limit + index}
mutated ${first_parsed_debug.count_mutated + mut_index}
interval_top ${first_parsed_debug.count_interval_top}
interval_down ${first_parsed_debug.count_interval_down}`,
  );

  let params = await page.evaluate(() => {
    let params = new URLSearchParams(document.location.search);
    let direction = params.get("direction");
    let time = params.get("time");
    let scroll = params.get("scroll");
    let tags = params.get("tags");
    let img_count = params.get("img_count");
    return `direction=${direction}&scroll=${scroll}&tags=${tags}&img_count=${img_count}`;
  });
  let expected_tags = text == "" ? "null" : text;
  expect(params).toBe(
    `direction=down&scroll=null&tags=${expected_tags}&img_count=${img_count}`,
  );
};

let get_parsed_debug_state_fn = async (page) => {
  let debug = await page.evaluate(async () => wasm_bindgen.get_debug_state());
  console.log(`e2e DEBUG STATE ${JSON.stringify(debug, null, 2)}`);
  let gallery_param_limit = get_manual_data("set_gallery_param_limit", debug).map((v) => Number(v.data));

  return {
    count_interval_top: get_manual_data("gallery_interval_top_triggered", debug).length,
    count_interval_down: get_manual_data("gallery_interval_down_triggered", debug).length,
    count_mutated: get_manual_data("gallery_mutated", debug).length,
    count_init: get_manual_data("gallery_init_executed", debug).length,
    count_reset: get_manual_data("gallery_reset_executed", debug).length,
    count_anchor_selected: get_manual_data("anchor_selected", debug).length,
    count_scroll_corrected: get_manual_data("scroll_correction", debug).length,
    count_scroll_correction_reset: get_manual_data("scroll_correction_reset", debug).length,

    count_gallery_param_limit: gallery_param_limit.length,
    gallery_param_limit: gallery_param_limit,
    gallery_items: get_signal_data("gallery_api_items", debug),
    anchor_last: get_signal_data("anchor_last", debug),
    anchor_first: get_signal_data("anchor_first", debug),

  };
};

let get_manual_data = (label, debug_state) => {
  let data = debug_state.manual_data
    .filter((v) => v.label == label)

  console.log(
    `e2e DEBUG STATE ${label} ${JSON.stringify(data, null, 2)}`,
  );
  return data;
};

let get_signal_data = (label, debug_state) => {
  let data = debug_state.signal_data
    .filter((v) => v.label == label)
    .map((v) => v.data.map((v)=>JSON.parse(v)) )

  console.log(
    `e2e DEBUG STATE ${label} ${JSON.stringify(data, null, 2)}`,
  );

  return data;
};

let get_signal_data_latest = (signal_data)=>{
  let data = signal_data[signal_data.length - 1];
  data = data[data.length - 1];
  return data;
};

let scroll_down_fn = async (
  page,
  gallery,
  offset,
  page_offset_y,
  scroll_iter_index,
) => {
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
