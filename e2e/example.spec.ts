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

  let anchor_first_all_fn = (debug_state) => {
    // console.log(`e2e DEBUG STATE ANCHOR FIRST ${JSON.stringify(debug_state, null, 2)}`);
    let anchor_first = debug_state.signal_data.filter(
      (v) => v.label == "anchor_first",
    );
    expect(anchor_first.length).toBe(1);
    console.log(`e2e DEBUG STATE1 ANCHOR FIRST ${JSON.stringify(anchor_first, null, 2)}`);
    anchor_first = anchor_first[0].data;
    return anchor_first;
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

  let anchor_first_locator = (anchor_first_arr) => {
    let o = anchor_first_arr[anchor_first_arr.length - 1];
    o = JSON.parse(o);
    console.log(`e2e anchor_first_locator ${JSON.stringify(o, null, 2)}`);
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
    let debug_state = await get_debug_state_fn();

    let anchor_all = anchor_first_all_fn(debug_state);
    let anchor = anchor_first_locator(anchor_all);

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

  let debug3 = await get_debug_state_fn(page);
  let count = get_scroll_correction_reset_count_fn(debug3);
  expect(count).toBe(0);
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
  await scroll_down_fn();
  await page.waitForTimeout(1000);

  let top_before = await gallery.evaluate((elm) => elm.scrollTop);
  let params = await page.evaluate(() => {
    let params = new URLSearchParams(document.location.search);
    let direction = params.get("direction");
    let time = params.get("time");
    let scroll = params.get("scroll");
    return `direction=${direction}&time=${time}&scroll=${scroll}`;
  });

  let debug = await get_debug_state_fn();
  let gallery_items = get_gallery_items(debug);
  let time = gallery_items[0].created_at;
  // expect(time).toBe("1777452967556484570");
  // 1777452967556484570
  expect(params).toBe(
    // `scroll=${top_before}`,
    `direction=down&time=${time}&scroll=${top_before}`,
  );

  // expect(JSON.stringify(params, null, 2)).toBe("q");
  await page.reload();
  let first_elm_id_after = await page
    .locator('[id="gallery"] > a')
    .first()
    .evaluate((elm) => elm.id);

  let top_after = await gallery.evaluate((elm) => elm.scrollTop);
  // TODO compare items too perhaps, should be same ones
  expect(top_before).toBe(top_after);

  let debug3 = await get_debug_state_fn(page);
  let count = get_scroll_correction_reset_count_fn(debug3);
  expect(count).toBe(0);
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
  let debug = await get_debug_state_fn(page);
  let first_item_time = get_gallery_items(debug)[0].created_at;

  await scroll_down_fn(page, gallery, offset, page_offset_y, scroll_iter_index);
  // await page.waitForTimeout(1000);
  await page.locator('[id="gallery"] > a').first().waitFor();

  // let params = await page.evaluate(() => {
  //   let params = new URLSearchParams(document.location.search);
  //   let direction = params.get("direction");
  //   let time = params.get("time");
  //   let scroll = params.get("scroll");
  //   return `direction=${direction}&time=${time}&scroll=${scroll}`;
  // });

  let banner = page.locator('[id="banner"]');
  await banner.click();
  await page.waitForTimeout(1000);
  let first_elm_id_after = await page
    .locator('[id="gallery"] > a')
    .first()
    .evaluate((elm) => elm.id);
  let debug2 = await get_debug_state_fn(page);

  expect(first_elm_id_before).toBe(first_elm_id_after);
  // await page.locator('[id="gallery"] > a').first().waitFor();

  let params2 = await page.evaluate(() => {
    let params = new URLSearchParams(document.location.search);
    let direction = params.get("direction");
    let time = params.get("time");
    let scroll = params.get("scroll");
    return `direction=${direction}&time=${time}&scroll=${scroll}`;
  });

  expect(params2).toBe(`direction=down&time=${first_item_time}&scroll=null`);
  let debug3 = await get_debug_state_fn(page);
  let count = get_scroll_correction_reset_count_fn(debug3);
  expect(count).toBe(1);
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
  // await gallery_search(page, first_debug, 9, 16, "ONE", "3");
  // SCROLLLLLLLLLLLLLLLLLLLL 0
  
});

test("gallery_search_from_diffrent_page", async ({ page }) => {
  await page.goto("http://localhost:3000/login");

  await page.locator('[id="search"]').fill("");
  await page.locator('[id="search"]').focus();
  await page.keyboard.press('Enter');

  let first_elm_id_before = await page
    .locator('[id="gallery"] > a')
    .first()
    .evaluate((elm) => elm.id);
});

let get_debug_state_fn = async (page) => {
    let debug_state = await page.evaluate(async () =>
      wasm_bindgen.get_debug_state(),
    );
    console.log(`e2e DEBUG STATE ${JSON.stringify(debug_state, null, 2)}`);
    return debug_state;
};


let gallery_search = async (page, first_parsed_debug, index, mut_index, text, img_count) => {
    await page.locator('[id="search"]').fill(text);
    await page.locator('[id="search"]').focus();
    await page.keyboard.press('Enter');
    await page.waitForTimeout(1000);

    let new_debug = await get_parsed_debug_state_fn(page);

    expect(
`init_executed ${new_debug.count_init}
reset_executed ${new_debug.count_reset}
param_limit ${new_debug.count_gallery_param_limit}
mutated ${new_debug.count_mutated}
interval_top ${new_debug.count_interval_top}
interval_down ${new_debug.count_interval_down}`)
.toBe(
`init_executed ${first_parsed_debug.count_init}
reset_executed ${first_parsed_debug.count_reset + index}
param_limit ${first_parsed_debug.count_gallery_param_limit + index}
mutated ${first_parsed_debug.count_mutated + mut_index}
interval_top ${first_parsed_debug.count_interval_top}
interval_down ${first_parsed_debug.count_interval_down}`);

  let params = await page.evaluate(() => {
    let params = new URLSearchParams(document.location.search);
    let direction = params.get("direction");
    let time = params.get("time");
    let scroll = params.get("scroll");
    let tags = params.get("tags");
    let img_count = params.get("img_count");
    return `direction=${direction}&scroll=${scroll}&tags=${tags}&img_count=${img_count}`;
  });
  let expected_tags =  text == "" ? "null" : text;
  expect(params).toBe(`direction=down&scroll=null&tags=${expected_tags}&img_count=${img_count}`);
};

let get_parsed_debug_state_fn = async (page) => {
    let debug = await page.evaluate(async () =>
      wasm_bindgen.get_debug_state(),
    );
    console.log(`e2e DEBUG STATE ${JSON.stringify(debug, null, 2)}`);

    let gallery_param_limit = get_gallery_param_limit_fn(debug);
    // let first_interval_top_count = get_interval_top_triggered_count_fn(debug);
    // let first_interval_down_count = get_interval_down_triggered_count_fn(debug);
  // let first_mutated_count = mutated_count_fn(debug);
  // let first_init_executed_count = gallery_init_executed_count_fn(debug);
  // let first_reset_executed_count = gallery_reset_executed_count_fn(debug);

    return {
        count_interval_top: get_interval_top_triggered_count_fn(debug),
        count_interval_down: get_interval_down_triggered_count_fn(debug),
        count_mutated: mutated_count_fn(debug),
        count_init: gallery_init_executed_count_fn(debug),
        count_reset: gallery_reset_executed_count_fn(debug),
        count_gallery_param_limit: gallery_param_limit.length,
        gallery_param_limit: gallery_param_limit,
    };
};


let gallery_init_executed_count_fn = (debug_state) => {
    let data = debug_state.manual_data.filter(
      (v) => v.label == "gallery_init_executed",
    );
    console.log(
      `e2e DEBUG STATE gallery_init_executed_count_fn ${JSON.stringify(data.length)}`,
    );
    return data.length;
};

let gallery_reset_executed_count_fn = (debug_state) => {
    let data = debug_state.manual_data.filter(
      (v) => v.label == "gallery_reset_executed",
    );
    console.log(
      `e2e DEBUG STATE gallery_reset_executed_count_fn ${JSON.stringify(data.length)}`,
    );
    return data.length;
};

let mutated_count_fn = (debug_state) => {
    let gallery_mutated = debug_state.manual_data.filter(
      (v) => v.label == "gallery_mutated",
    );
    console.log(
      `e2e DEBUG STATE GALLERY_MUTATED ${JSON.stringify(gallery_mutated.length)}`,
    );
    return gallery_mutated.length;
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

let get_scroll_correction_reset_count_fn = (debug_state) => {
    let data = debug_state.manual_data.filter(
      (v) => v.label == "scroll_correction_reset",
    );
    let count = data.length;
    console.log(
      `e2e DEBUG STATE scroll_correction_reset_count ${count}`,
    );
    return count;
};

let get_interval_top_triggered_count_fn = (debug_state) => {
    let data = debug_state.manual_data.filter(
      (v) => v.label == "gallery_interval_top_triggered",
    );
    let count = data.length;
    console.log(
      `e2e DEBUG STATE get_interval_top_triggered_count_fn ${count}`,
    );
    return count;
};

let get_interval_down_triggered_count_fn = (debug_state) => {
    let data = debug_state.manual_data.filter(
      (v) => v.label == "gallery_interval_down_triggered",
    );
    let count = data.length;
    console.log(
      `e2e DEBUG STATE get_interval_down_triggered_count_fn ${count}`,
    );
    return count;
};

let get_gallery_param_limit_fn = (debug_state) => {
    let data = debug_state.manual_data.filter(
      (v) => v.label == "set_gallery_param_limit",
    ).map((v) => Number(v.data) );
    // let count = data.length;
    console.log(
      `e2e DEBUG STATE set_gallery_param_limit ${JSON.stringify(data, null, 2)}`,
    );
    return data;
};

let scroll_down_fn = async (page, gallery, offset, page_offset_y, scroll_iter_index) => {
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

