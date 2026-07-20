import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

const root = process.cwd();

describe("offline delivery", () => {
  it("does not reference remote runtime assets", () => {
    const files = ["index.html", "src/styles.css", ...readdirSync(join(root, "src"), { recursive: true }).filter((file) => /\.(ts|tsx)$/.test(String(file))).map((file) => join("src", String(file)))];
    for (const file of files) {
      const contents = readFileSync(join(root, file), "utf8");
      expect(contents, file).not.toMatch(/https?:\/\//);
    }
  });
});
