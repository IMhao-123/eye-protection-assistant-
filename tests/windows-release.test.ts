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
      mainBinaryName?: string;
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
    expect(config.mainBinaryName).toBe("护眼助手");
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
    const packageJson = JSON.parse(
      readFileSync(join(root, "package.json"), "utf8"),
    ) as {
      scripts: Record<string, string>;
    };
    const workflow = readFileSync(
      join(root, ".github", "workflows", "ci.yml"),
      "utf8",
    );
    expect(workflow).toContain("runs-on: windows-latest");
    expect(workflow).toContain("npm run verify:windows");
    expect(workflow).toContain("npm run package:windows");
    expect(workflow).toContain("Smoke test installed app");
    expect(workflow).toContain("Get-FileHash");
    expect(workflow).toContain("uninstall.exe");
    expect(workflow).toContain("*.sha256");
    expect(packageJson.scripts["verify:windows"]).toContain(
      "npm run test:rust:core",
    );
    expect(packageJson.scripts["test:rust:core"]).toContain(
      "-p eye-care-core",
    );
    expect(packageJson.scripts["verify:windows"]).toContain(
      "npm run test:rust:windows-compile",
    );
    expect(packageJson.scripts["test:rust:windows-compile"]).toContain(
      "--no-run",
    );
    expect(workflow).toContain("$first.Refresh()");
    expect(workflow).toContain(
      'if ($first.HasExited) { throw "First instance exited after second launch" }',
    );
  });

  it("packages a valid PNG tray resource", () => {
    const trayIcon = readFileSync(
      join(root, "src-tauri", "icons", "trayTemplate.png"),
    );
    expect(Array.from(trayIcon.subarray(0, 8))).toEqual([
      0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a,
    ]);
  });

  it("restores a minimized main window before focusing it", () => {
    const backend = readFileSync(
      join(root, "src-tauri", "src", "lib.rs"),
      "utf8",
    );
    expect(backend).toContain("main.unminimize()");
  });

  it("applies tested window policies in the desktop builders", () => {
    const backend = readFileSync(
      join(root, "src-tauri", "src", "lib.rs"),
      "utf8",
    );
    expect(backend).toContain("let widget_policy = widget_window_policy()");
    expect(backend).toContain(
      ".skip_taskbar(widget_policy.skip_taskbar)",
    );
    expect(backend).toContain("let break_policy = break_window_policy()");
    expect(backend).toContain(".skip_taskbar(break_policy.skip_taskbar)");
    expect(backend).not.toContain(".always_on_top(true)");
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
