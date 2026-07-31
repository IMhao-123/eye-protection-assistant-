import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

const root = process.cwd();

describe("Windows release contract", () => {
  it("builds one Windows x64 NSIS installer", () => {
    const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8")) as {
      scripts: Record<string, string>;
    };
    const config = JSON.parse(
      readFileSync(join(root, "src-tauri", "tauri.conf.json"), "utf8"),
    ) as {
      app: { macOSPrivateApi?: boolean };
      bundle: {
        targets: string[];
        longDescription: string;
        icon: string[];
        windows?: {
          webviewInstallMode?: { type?: string };
          nsis?: { installMode?: string; languages?: string[] };
        };
      };
    };

    expect(packageJson.scripts["package:windows"]).toBe(
      "tauri build --bundles nsis",
    );
    expect(config.app.macOSPrivateApi).not.toBe(true);
    expect(config.bundle.targets).toEqual(["nsis"]);
    expect(config.bundle.icon).toContain("icons/icon.ico");
    expect(config.bundle.longDescription).toContain("Windows");
    expect(config.bundle.windows?.webviewInstallMode?.type).toBe(
      "downloadBootstrapper",
    );
    expect(config.bundle.windows?.nsis?.installMode).toBe("currentUser");
    expect(config.bundle.windows?.nsis?.languages).toEqual([
      "SimpChinese",
      "English",
    ]);
  });

  it("runs Windows verification and NSIS packaging in CI", () => {
    const workflow = readFileSync(
      join(root, ".github", "workflows", "ci.yml"),
      "utf8",
    );
    expect(workflow).toContain("runs-on: windows-latest");
    expect(workflow).toContain("npm run verify");
    expect(workflow).toContain("npm run package:windows");
    expect(workflow).toContain("Smoke test installed app");
    expect(workflow).toContain("Get-FileHash");
    expect(workflow).toContain("uninstall.exe");
    expect(workflow).toContain("*.sha256");
  });

  it("restores a minimized main window before focusing it", () => {
    const backend = readFileSync(
      join(root, "src-tauri", "src", "lib.rs"),
      "utf8",
    );
    expect(backend).toContain("main.unminimize()");
  });

  it("does not track dependency caches, generated schemas, or release binaries", () => {
    const trackedFiles = execFileSync("git", ["ls-files"], {
      cwd: root,
      encoding: "utf8",
    })
      .trim()
      .split("\n")
      .filter(Boolean);
    expect(trackedFiles).not.toContain("src-tauri/gen/schemas/desktop-schema.json");
    expect(trackedFiles).not.toContain("src-tauri/gen/schemas/macOS-schema.json");
    expect(trackedFiles).not.toContain("src-tauri/gen/schemas/acl-manifests.json");
    expect(trackedFiles).not.toContain("src-tauri/gen/schemas/capabilities.json");
    expect(
      trackedFiles.filter((file) =>
        /(^|\/)(node_modules|dist|target)(\/|$)|\.(app|dmg|exe|msi)$/i.test(
          file,
        ),
      ),
    ).toEqual([]);
  });
});
