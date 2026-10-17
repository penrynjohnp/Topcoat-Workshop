import { spawnSync } from "node:child_process";

for (const target of ["lab07", "lab08", "slipway"]) {
  const result = spawnSync(
    process.execPath,
    ["node_modules/@playwright/test/cli.js", "test"],
    {
      env: { ...process.env, BROWSER_TARGET: target },
      stdio: "inherit",
    },
  );

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}
