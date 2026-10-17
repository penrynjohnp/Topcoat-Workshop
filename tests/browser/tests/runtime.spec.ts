import { expect, test } from "@playwright/test";

test("runtime expressions and navigation work without a document reload", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveTitle("Slipway");
  await expect(page.locator("[data-inspection-summary]")).toContainText("Scheduled");

  await page.getByRole("button", { name: "Advance", exact: true }).click();
  await expect(page.locator("[data-inspection-summary]")).toContainText("turn 2");

  await page.locator('[data-command="increment-shared-count"]').click();
  await expect(page.getByRole("button", { name: /Shared count:/ })).toContainText("1");

  await page.evaluate(() => {
    (window as typeof window & { workshopNavigationProbe?: string }).workshopNavigationProbe =
      "kept";
  });
  const navigationCount = await page.evaluate(
    () => performance.getEntriesByType("navigation").length,
  );

  const berthLink = page.getByRole("link", { name: "Berths", exact: true });
  await expect(berthLink).toHaveAttribute("data-topcoat-link", "intent");
  await berthLink.click();

  await expect(page).toHaveURL(/\/berths$/);
  await expect(page.locator("ul.berths > li[data-berth='a1']")).toBeVisible();
  await expect(page.getByRole("button", { name: /Shared count:/ })).toContainText("1");
  await expect(
    page.getByRole("link", { name: "Berths", exact: true }),
  ).toHaveAttribute("aria-current", "page");
  expect(
    await page.evaluate(
      () =>
        (window as typeof window & { workshopNavigationProbe?: string })
          .workshopNavigationProbe,
    ),
  ).toBe("kept");
  expect(await page.evaluate(() => performance.getEntriesByType("navigation").length)).toBe(
    navigationCount,
  );

  await expect(page.getByRole("link", { name: "Admin", exact: true })).toHaveAttribute(
    "data-topcoat-link",
    "never",
  );
});
