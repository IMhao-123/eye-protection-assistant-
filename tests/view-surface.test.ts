import { describe, expect, it } from "vitest";
import { applyDocumentSurface, resolveViewMode } from "../src/viewSurface";

describe("window view surface", () => {
  it.each([
    ["?view=widget", "widget"],
    ["?view=break&monitor=1", "break"],
    ["", "main"],
    ["?view=unknown", "main"],
  ] as const)("resolves %s to %s", (search, expected) => {
    expect(resolveViewMode(search)).toBe(expected);
  });

  it("marks the document before rendering the widget", () => {
    applyDocumentSurface("widget", document);
    expect(document.documentElement.dataset.view).toBe("widget");
  });
});
