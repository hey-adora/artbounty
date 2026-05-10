import { test, expect } from "@playwright/test";

test("title", async ({ page }) => {
  let viewport_height = 915;
  // viewport: { width: 412, height: 915 },
  await page.goto("http://localhost:3000");
  await page.waitForTimeout(5000);
  await expect(page.locator('[id="5idoghr47bvsajsi5izx-link"]')).toBeVisible();

  let gallery = page.locator('[id="gallery"]');

  let item0_id = await gallery.evaluate((elm) => elm.firstElementChild.id);
  let item0_id_str = `[id="${item0_id}"]`;
  let item1_id = await gallery.evaluate((elm) => elm.lastElementChild.id);
  let item1_id_str = `[id="${item1_id}"]`;
  let item0 = page.locator(item0_id_str);
  let item1 = page.locator(item1_id_str);
  // let item0 = gallery.lastElementChild gallery.evaluate((elm) => elm.scrollHeight);

  let gallery_scrollheight = await gallery.evaluate((elm) => elm.scrollHeight);
  let gallery_height = await gallery.evaluate((elm) => elm.clientHeight);
  let item0_top = await item0.evaluate(
    (elm) => elm.getBoundingClientRect().top,
  );
  let item1_top = await item1.evaluate(
    (elm) => elm.getBoundingClientRect().top,
  );

  let offset = 1;
  let scroll_by = item1_top - (item0_top + gallery_height + offset); // one pixel offset to not load more elements

  // expect(scroll_by).toBe(2260);

  // let gallery_scrolltop = gallery?.scrollTop;
  // let gallery_scrollheight = gallery?.scrollHeight;

  // expect(gallery_scrolltop).toBe();
  await page.mouse.move(200, 400);

  await page.mouse.wheel(0, scroll_by);
  await page.waitForTimeout(3000);

  let item1_top2 = await item1.evaluate(
    (elm) => elm.getBoundingClientRect().top,
  );

  await page.mouse.wheel(0, offset);
  await page.waitForTimeout(3000);

  let item1_top3 = await item1.evaluate(
    (elm) => elm.getBoundingClientRect().top,
  );

  {
    let expected_top = item1_top2 + offset;
    let expected_left = expected_top - (expected_top % 5);
    let expected_right = item1_top3 - (item1_top3 % 5);
    console.log(`E2E ${expected_left} == ${expected_right}`);
    expect(expected_left).toBe(expected_right);
  }

  let item2_id = await gallery.evaluate((elm) => elm.firstElementChild.id);
  let item2_id_str = `[id="${item2_id}"]`;
  let item3_id = await gallery.evaluate((elm) => elm.lastElementChild.id);
  let item3_id_str = `[id="${item3_id}"]`;
  let item2 = page.locator(item2_id_str);
  let item3 = page.locator(item3_id_str);
  
  let item2_top = await item2.evaluate(
    (elm) => elm.getBoundingClientRect().top,
  );
  let item3_top = await item3.evaluate(
    (elm) => elm.getBoundingClientRect().top,
  );

  offset = 1;
  scroll_by = item3_top - (item0_top + gallery_height + offset); // one pixel offset to not load more elements
  console.log(`E2E scroll by ${scroll_by}`);


  await page.mouse.wheel(0, scroll_by);
  await page.waitForTimeout(3000);

  let item3_top2 = await item3.evaluate(
    (elm) => elm.getBoundingClientRect().top,
  );

  await page.mouse.wheel(0, offset);
  await page.waitForTimeout(3000);

  let item3_top3 = await item3.evaluate(
    (elm) => elm.getBoundingClientRect().top,
  );

  
  {
    let expected_top = item3_top2 + offset;
    let expected_left = expected_top - (expected_top % 5);
    let expected_right = item3_top3 - (item3_top3 % 5);
    console.log(`E2E ${expected_left} == ${expected_right}`);
    expect(expected_left).toBe(expected_right);
  }


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
