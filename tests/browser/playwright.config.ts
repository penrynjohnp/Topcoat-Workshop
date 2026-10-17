import { defineConfig } from "@playwright/test";
import { fileURLToPath } from "node:url";

const repositoryRoot = fileURLToPath(new URL("../..", import.meta.url));
const target = process.env.BROWSER_TARGET;
const packages = {
  lab07: { package: "lab07-solution", binary: "lab07-solution" },
  lab08: { package: "lab08-solution", binary: "lab08-solution" },
  slipway: { package: "slipway-capstone", binary: "slipway-capstone" },
} as const;

if (target !== "lab07" && target !== "lab08" && target !== "slipway") {
  throw new Error(`Unsupported browser-test target: ${target ?? "(unset)"}`);
}

const selected = packages[target];
const runtimeTest = target === "lab07" ? "**/*.spec.ts" : "**/navigation.spec.ts";

export default defineConfig({
  testDir: "./tests",
  fullyParallel: false,
  reporter: "list",
  testMatch: runtimeTest,
  use: {
    baseURL: "http://127.0.0.1:3000",
    browserName: "chromium",
    headless: true,
    trace: "retain-on-failure",
  },
  webServer: {
    command:
      `cargo build -p ${selected.package} && topcoat asset bundle -p ${selected.package} && target/debug/${selected.binary}`,
    cwd: repositoryRoot,
    url: "http://127.0.0.1:3000",
    reuseExistingServer: false,
    timeout: 120_000,
    env:
      target === "slipway"
        ? { SLIPWAY_DATABASE_URL: "sqlite::memory:" }
        : {},
  },
});
