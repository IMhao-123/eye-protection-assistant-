import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const css = readFileSync(resolve(process.cwd(), "src/styles.css"), "utf8");

describe("color scheme tokens", () => {
  it("separates palette values from the light and dark theme mapping", () => {
    expect(css).toContain("--palette-light-bg:");
    expect(css).toContain("--palette-dark-bg:");
    expect(css).toContain("--bg: var(--palette-light-bg)");
    expect(css).toContain("--bg: var(--palette-dark-bg)");
  });

  it("defines complete morning lake and graphite lime palettes", () => {
    for (const scheme of ["morning_lake", "graphite_lime"]) {
      const selector = `:root[data-color-scheme="${scheme}"]`;
      const start = css.indexOf(selector);
      expect(start).toBeGreaterThan(-1);
      const block = css.slice(start, css.indexOf("}", start));
      for (const token of [
        "--palette-light-bg:",
        "--palette-dark-bg:",
        "--palette-light-accent-secondary:",
        "--palette-dark-accent-secondary:",
        "--palette-light-widget-bg:",
        "--palette-dark-widget-bg:",
        "--palette-light-break-bg:",
        "--palette-dark-break-bg:",
        "--palette-light-control-knob:",
        "--palette-dark-control-knob:",
      ]) {
        expect(block).toContain(token);
      }
    }
  });

  it("uses a dark graphite knob against the bright lime switch", () => {
    const start = css.indexOf(':root[data-color-scheme="graphite_lime"]');
    const block = css.slice(start, css.indexOf("}", start));
    expect(block).toContain("--palette-dark-accent: #a9d84e");
    expect(block).toContain("--palette-dark-control-knob: #17200f");
  });

  it("defines the complete default mist blue coral palette", () => {
    const start = css.indexOf(':root[data-color-scheme="mist_blue_coral"]');
    expect(start).toBeGreaterThan(-1);
    const block = css.slice(start, css.indexOf("}", start));
    for (const token of [
      "--palette-light-primary-bg: #327da8",
      "--palette-dark-primary-bg: #74b8df",
      "--palette-light-accent-secondary: #b6534b",
      "--palette-dark-accent-secondary: #e58a82",
      "--palette-light-widget-bg:",
      "--palette-dark-widget-bg:",
      "--palette-light-break-bg:",
      "--palette-dark-break-bg:",
    ]) {
      expect(block).toContain(token);
    }
  });

  it("defines the complete porcelain forest palette without using yellow for body text", () => {
    const start = css.indexOf(':root[data-color-scheme="porcelain_forest"]');
    expect(start).toBeGreaterThan(-1);
    const block = css.slice(start, css.indexOf("}", start));
    for (const token of [
      "--palette-light-primary-bg: #347445",
      "--palette-dark-primary-bg: #82bd8c",
      "--palette-light-accent-secondary: #8a6508",
      "--palette-dark-accent-secondary: #e7c766",
      "--palette-light-widget-bg:",
      "--palette-dark-widget-bg:",
      "--palette-light-break-bg:",
      "--palette-dark-break-bg:",
      "--palette-light-muted: #5f6b61",
      "--palette-dark-muted: #adb9ae",
    ]) {
      expect(block).toContain(token);
    }
  });
});
