import { test, expect } from "@playwright/test";

export let login = async (page)=>{
  await page.goto("http://localhost:3000/login");

  await page.locator('[id="email"]').fill("prime1@heyadora.com");
  await page.locator('[id="password"]').fill("prime1@heyadora.com");
  await page.locator('[id="login_btn"]').click();
};

export let gallery_search = async (
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

export let get_parsed_debug_state_fn = async (page) => {
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
    post_description_mutation: get_manual_data("post_description_mutation", debug),
    gallery_items: get_signal_data("gallery_api_items", debug),
    anchor_last: get_signal_data("anchor_last", debug),
    anchor_first: get_signal_data("anchor_first", debug),

  };
};

export let get_manual_data = (label, debug_state) => {
  let data = debug_state.manual_data
    .filter((v) => v.label == label)
    .map((v)=>v.data)

  console.log(
    `e2e DEBUG STATE ${label} ${JSON.stringify(data, null, 2)}`,
  );
  return data;
};

export let get_signal_data = (label, debug_state) => {
  let data = debug_state.signal_data
    .filter((v) => v.label == label)
    .map((v) => v.data.map((v)=>JSON.parse(v)) )

  console.log(
    `e2e DEBUG STATE ${label} ${JSON.stringify(data, null, 2)}`,
  );

  return data;
};

export let get_signal_data_latest = (signal_data)=>{
  let data = signal_data[signal_data.length - 1];
  data = data[data.length - 1];
  return data;
};

export let scroll_down_fn = async (
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
