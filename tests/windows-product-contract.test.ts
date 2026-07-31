import { readFileSync, readdirSync } from "node:fs";
import { extname, join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  AVAILABLE_COLOR_SCHEMES,
  DEFAULT_SETTINGS,
} from "../src/types";

const root = process.cwd();

function sourceFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? sourceFiles(path) : [path];
  });
}

describe("Windows product contract", () => {
  it("keeps the complete visual and settings contract", () => {
    expect(AVAILABLE_COLOR_SCHEMES).toEqual([
      "original",
      "morning_lake",
      "graphite_lime",
      "mist_blue_coral",
      "porcelain_forest",
    ]);
    expect(DEFAULT_SETTINGS).toMatchObject({
      language: "system",
      theme: "system",
      colorScheme: "mist_blue_coral",
      skipConfirmation: true,
      notificationEnabled: true,
    });
  });

  it("keeps runtime source offline and free of tracking services", () => {
    const packageJson = JSON.parse(
      readFileSync(join(root, "package.json"), "utf8"),
    ) as {
      dependencies: Record<string, string>;
      scripts: Record<string, string>;
    };
    const config = JSON.parse(
      readFileSync(join(root, "src-tauri", "tauri.conf.json"), "utf8"),
    ) as { app: { security: { csp: string } } };
    const runtimeSource = sourceFiles(join(root, "src"))
      .filter((path) => [".ts", ".tsx", ".css"].includes(extname(path)))
      .map((path) => readFileSync(path, "utf8"))
      .join("\n");

    expect(packageJson.scripts["verify:windows-contract"]).toBe(
      "vitest run tests/windows-release.test.ts tests/windows-interaction.test.ts tests/windows-product-contract.test.ts",
    );
    expect(Object.keys(packageJson.dependencies)).not.toEqual(
      expect.arrayContaining([
        "axios",
        "firebase",
        "posthog-js",
        "@sentry/react",
        "mixpanel-browser",
      ]),
    );
    expect(runtimeSource).not.toMatch(/https?:\/\//i);
    expect(config.app.security.csp.match(/https?:\/\/[^\s;]+/gi)).toEqual([
      "http://ipc.localhost",
    ]);
  });
});
