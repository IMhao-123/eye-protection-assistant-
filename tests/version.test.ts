import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

const root = process.cwd();
const expectedVersion = "0.3.0";

describe("release version", () => {
  it("keeps npm, Cargo, and Tauri metadata aligned", () => {
    const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8")) as {
      version: string;
    };
    const packageLock = JSON.parse(readFileSync(join(root, "package-lock.json"), "utf8")) as {
      version: string;
      packages: Record<string, { version?: string }>;
    };
    const tauriConfig = JSON.parse(
      readFileSync(join(root, "src-tauri", "tauri.conf.json"), "utf8"),
    ) as { version: string };
    const cargoManifest = readFileSync(join(root, "src-tauri", "Cargo.toml"), "utf8");
    const cargoVersion = cargoManifest.match(
      /^\[package\][\s\S]*?^version = "([^"]+)"/m,
    )?.[1];

    expect(packageJson.version).toBe(expectedVersion);
    expect(packageLock.version).toBe(expectedVersion);
    expect(packageLock.packages[""]?.version).toBe(expectedVersion);
    expect(cargoVersion).toBe(expectedVersion);
    expect(tauriConfig.version).toBe(expectedVersion);
  });
});
