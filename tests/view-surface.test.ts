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
    const root = document.createElement("div");
    root.id = "root";
    document.body.append(root);
    applyDocumentSurface("widget", document);
    expect(document.documentElement.dataset.view).toBe("widget");
    expect(document.documentElement.style.backgroundColor).toBe("transparent");
    expect(document.body.style.backgroundColor).toBe("transparent");
    expect(root.style.backgroundColor).toBe("transparent");
    root.remove();
  });
});
