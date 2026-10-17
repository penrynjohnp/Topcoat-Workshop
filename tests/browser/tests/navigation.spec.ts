import { expect, test } from "@playwright/test";

test("runtime navigation preserves shared state and updates the current page", async ({
  page,
}) => {
  await page.goto("/");
  await page.getByRole("button", { name: /Shared count:/ }).click();

  const navigationCount = await page.evaluate(
    () => performance.getEntriesByType("navigation").length,
  );
  const berthLink = page.getByRole("link", { name: "Berths", exact: true });
  await expect(berthLink).toHaveAttribute("data-topcoat-link", "intent");
  await berthLink.click();

  await expect(page).toHaveURL(/\/berths$/);
  await expect(page.getByRole("button", { name: /Shared count:/ })).toContainText("1");
  await expect(berthLink).toHaveAttribute("aria-current", "page");
  expect(await page.evaluate(() => performance.getEntriesByType("navigation").length)).toBe(
    navigationCount,
  );

  const requests: string[] = [];
  page.on("request", (request) => requests.push(new URL(request.url()).pathname));
  const adminLink = page.getByRole("link", { name: "Admin", exact: true });
  await expect(adminLink).toHaveAttribute("data-topcoat-link", "never");
  await adminLink.hover();
  await page.waitForTimeout(500);
  expect(requests).not.toContain("/admin");
});

test("ordinary href navigation remains available without JavaScript", async ({ browser }) => {
  const context = await browser.newContext({ javaScriptEnabled: false });
  const page = await context.newPage();
  try {
    await page.goto("/");
    await expect(page.getByRole("link", { name: "Berths", exact: true })).toHaveAttribute(
      "href",
      "/berths",
    );
    await page.getByRole("link", { name: "Berths", exact: true }).click();
    await expect(page).toHaveURL(/\/berths$/);
  } finally {
    await context.close();
  }
});
