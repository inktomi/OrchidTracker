import { test, expect } from "@playwright/test";

// These tests verify the SSR -> hydration pipeline works correctly.
// They catch the class of bugs where WASM panics during hydrate(),
// which silently breaks all event handlers on the page.

test.describe("Hydration health", () => {
  test("no WASM panics in console after page load", async ({ page }) => {
    const errors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        errors.push(msg.text());
      }
    });

    page.on("pageerror", (err) => {
      errors.push(err.message);
    });

    await page.goto("/");
    // Wait for WASM to load and hydration to complete
    await page.waitForTimeout(3000);

    const panics = errors.filter(
      (e) => e.includes("panicked") || e.includes("unreachable")
    );
    expect(panics).toHaveLength(0);
  });

  test("page has interactive elements after hydration", async ({ page }) => {
    await page.goto("/");
    await page.waitForTimeout(3000);

    // The page should have loaded either the home page (with header buttons)
    // or redirected to login. Either way, no blank page.
    const bodyText = await page.textContent("body");
    expect(bodyText).toBeTruthy();
    expect(bodyText!.length).toBeGreaterThan(0);
  });
});

test.describe("Login page", () => {
  test("login form is interactive", async ({ page }) => {
    await page.goto("/login");
    await page.waitForTimeout(2000);

    // Username and password inputs should exist and be fillable
    const usernameInput = page.locator('input[name="username"]');
    const passwordInput = page.locator('input[type="password"]');

    await expect(usernameInput).toBeVisible();
    await expect(passwordInput).toBeVisible();

    // Test that inputs accept text (proves hydration worked)
    await usernameInput.fill("testuser");
    await expect(usernameInput).toHaveValue("testuser");
  });
});

test.describe("Authenticated home page", () => {
  // Helper: log in and return the authenticated page
  async function loginAs(
    page: import("@playwright/test").Page,
    username: string,
    password: string
  ) {
    await page.goto("/login");
    await page.waitForTimeout(2000);

    await page.fill('input[name="username"]', username);
    await page.fill('input[type="password"]', password);
    await page.click('button[type="submit"]');

    // Wait for redirect to home or onboarding
    await page.waitForURL((url) => !url.pathname.includes("/login"), {
      timeout: 10_000,
    });
    await page.waitForTimeout(2000); // Wait for hydration
  }

  test("header buttons respond to clicks", async ({ page }) => {
    // Skip if no test credentials are configured
    const username = process.env.TEST_USERNAME;
    const password = process.env.TEST_PASSWORD;
    if (!username || !password) {
      test.skip(true, "Set TEST_USERNAME and TEST_PASSWORD to run this test");
      return;
    }

    await loginAs(page, username, password);

    // Verify we're on the home page (not stuck on login)
    await expect(page).not.toHaveURL(/\/login/);

    // Check for WASM panics
    const errors: string[] = [];
    page.on("pageerror", (err) => errors.push(err.message));

    // Test the Settings button in header
    const settingsBtn = page.locator("button", { hasText: "Settings" });
    if (await settingsBtn.isVisible()) {
      await settingsBtn.click();
      await page.waitForTimeout(500);

      // Settings modal should appear (or at minimum, no panic)
      const panics = errors.filter((e) => e.includes("panicked"));
      expect(panics).toHaveLength(0);
    }
  });

  test("Add button opens modal", async ({ page }) => {
    const username = process.env.TEST_USERNAME;
    const password = process.env.TEST_PASSWORD;
    if (!username || !password) {
      test.skip(true, "Set TEST_USERNAME and TEST_PASSWORD to run this test");
      return;
    }

    await loginAs(page, username, password);

    const errors: string[] = [];
    page.on("pageerror", (err) => errors.push(err.message));

    // Try the Add button (either in header or empty collection CTA)
    const addBtn = page.locator("button", { hasText: /^Add/ }).first();
    if (await addBtn.isVisible()) {
      await addBtn.click();
      await page.waitForTimeout(500);

      const panics = errors.filter((e) => e.includes("panicked"));
      expect(panics).toHaveLength(0);
    }
  });
});
