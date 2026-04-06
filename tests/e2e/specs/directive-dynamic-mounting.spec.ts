import { expect, test } from "@playwright/test";

test.describe("dynamic mounting in directives", () => {
  test("mounts dynamic children added by <each>", async ({ page }) => {
    await page.goto("/");

    const eachSection = page.locator("#each-test");

    await expect(eachSection.locator(".list-item")).toHaveCount(2);
    await expect(eachSection.locator(".list-item-mounted[data-mounted='true']")).toHaveCount(2);

    await eachSection.getByRole("button", { name: "Add item" }).click();

    await expect(eachSection.locator(".list-item")).toHaveCount(3);
    await expect(eachSection.locator(".list-item-mounted[data-mounted='true']")).toHaveCount(3);

    await eachSection.getByRole("button", { name: "Remove item" }).click();

    await expect(eachSection.locator(".list-item")).toHaveCount(2);
    await expect(eachSection.locator(".list-item-mounted[data-mounted='true']")).toHaveCount(2);
  });

  test("mounts dynamic child in active <if> branch only once", async ({ page }) => {
    await page.goto("/");

    await expect(page.locator("#details-closed")).toBeVisible();
    await expect(page.locator("#details-panel")).toBeHidden();

    await page.getByRole("button", { name: "Toggle details" }).click();

    await expect(page.locator("#details-panel")).toBeVisible();
    await expect(page.locator("#details-mounted-count")).toHaveText("mount-count: 1");

    await page.getByRole("button", { name: "Toggle details" }).click();
    await expect(page.locator("#details-closed")).toBeVisible();

    await page.getByRole("button", { name: "Toggle details" }).click();
    await expect(page.locator("#details-panel")).toBeVisible();
    await expect(page.locator("#details-mounted-count")).toHaveText("mount-count: 1");
  });

  test("handles nested <each> with inner <if> and mounts only active branches", async ({ page }) => {
    await page.goto("/");

    const nested = page.locator("#nested-each-if-test");

    await expect(nested.locator(".group-badge")).toHaveCount(1);
    await expect(nested.locator(".group-badge-mounted[data-mounted='true']")).toHaveCount(1);

    const initialVisibleItems = nested.locator(".list-item:visible");
    await expect(initialVisibleItems).toHaveCount(1);
    await expect(nested.locator(".list-item-mounted[data-mounted='true']:visible")).toHaveCount(1);

    await nested.getByRole("button", { name: "Add inner item to Team A" }).click();
    const visibleAfterAdd = await nested.locator(".list-item:visible").count();
    const mountedAfterAdd = await nested
      .locator(".list-item-mounted[data-mounted='true']:visible")
      .count();
    expect(visibleAfterAdd).toBeGreaterThanOrEqual(1);
    expect(mountedAfterAdd).toBe(visibleAfterAdd);

    await nested.getByRole("button", { name: "Toggle members visibility" }).click();
    const visibleAfterToggle = await nested.locator(".list-item:visible").count();
    const mountedAfterToggle = await nested
      .locator(".list-item-mounted[data-mounted='true']:visible")
      .count();
    expect(visibleAfterToggle).toBeGreaterThanOrEqual(1);
    expect(mountedAfterToggle).toBe(visibleAfterToggle);

    await nested.getByRole("button", { name: "Toggle members visibility" }).click();
    const visibleAfterRetoggle = await nested.locator(".list-item:visible").count();
    const mountedAfterRetoggle = await nested
      .locator(".list-item-mounted[data-mounted='true']:visible")
      .count();
    expect(visibleAfterRetoggle).toBeGreaterThanOrEqual(1);
    expect(mountedAfterRetoggle).toBe(visibleAfterRetoggle);

    await nested.getByRole("button", { name: "Clear all members" }).click();
    const visibleAfterClear = await nested.locator(".list-item:visible").count();
    const mountedAfterClear = await nested
      .locator(".list-item-mounted[data-mounted='true']:visible")
      .count();
    expect(visibleAfterClear).toBeLessThanOrEqual(visibleAfterRetoggle);
    expect(mountedAfterClear).toBe(visibleAfterClear);
  });

  test("mounts multiple dynamic components inside <if> -> <each> without duplicate remounts", async ({ page }) => {
    await page.goto("/");

    const conditional = page.locator("#if-each-multi-test");

    await expect(conditional.locator(".list-item")).toHaveCount(2);
    await expect(conditional.locator(".status-chip")).toHaveCount(2);
    await expect(conditional.locator(".list-item-mounted[data-mounted='true']")).toHaveCount(2);
    await expect(conditional.locator(".status-chip-mounted[data-mounted='true']")).toHaveCount(2);

    await conditional.getByRole("button", { name: "Add conditional item" }).click();
    await expect(conditional.locator(".list-item")).toHaveCount(3);
    await expect(conditional.locator(".status-chip")).toHaveCount(3);
    await expect(conditional.locator(".list-item-mounted[data-mounted='true']")).toHaveCount(3);
    await expect(conditional.locator(".status-chip-mounted[data-mounted='true']")).toHaveCount(3);

    await conditional.getByRole("button", { name: "Toggle conditional list" }).click();
    await expect(conditional.locator("#conditional-list-hidden")).toBeVisible();
    await expect(conditional.locator(".list-item")).toHaveCount(0);
    await expect(conditional.locator(".status-chip")).toHaveCount(0);

    await conditional.getByRole("button", { name: "Toggle conditional list" }).click();
    await expect(conditional.locator(".list-item")).toHaveCount(3);
    await expect(conditional.locator(".status-chip")).toHaveCount(3);
    await expect(conditional.locator(".list-item-mounted[data-mounted='true']")).toHaveCount(3);
    await expect(conditional.locator(".status-chip-mounted[data-mounted='true']")).toHaveCount(3);
  });

  test("updates <if> inside <each> and mounts only items with active branch", async ({ page }) => {
    await page.goto("/");

    const listWithConditional = page.locator("#each-if-inside-test");

    await expect(listWithConditional.locator(".list-conditional-row")).toHaveCount(3);
    await expect(listWithConditional.locator(".alert-tag:visible")).toHaveCount(3);
    await expect(
      listWithConditional.locator(".alert-tag-mounted[data-mounted='true']:visible"),
    ).toHaveCount(3);
    await expect(listWithConditional.locator(".no-alert:visible")).toHaveCount(3);

    await listWithConditional.getByRole("button", { name: "Toggle all alerts" }).click();
    await expect(listWithConditional.locator(".alert-tag:visible")).toHaveCount(3);
    await expect(
      listWithConditional.locator(".alert-tag-mounted[data-mounted='true']:visible"),
    ).toHaveCount(3);
    await expect(listWithConditional.locator(".no-alert:visible", { hasText: "no alert" })).toHaveCount(3);

    await listWithConditional.getByRole("button", { name: "Show all alerts" }).click();
    await expect(listWithConditional.locator(".alert-tag:visible")).toHaveCount(3);
    await expect(
      listWithConditional.locator(".alert-tag-mounted[data-mounted='true']:visible"),
    ).toHaveCount(3);
    await expect(listWithConditional.locator(".no-alert:visible", { hasText: "no alert" })).toHaveCount(3);
  });
});
