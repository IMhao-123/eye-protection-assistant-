import { describe, expect, it } from "vitest";
import {
  isEditingTarget,
  resolveKeyboardCommand,
} from "../src/keyboardShortcuts";

describe("Windows keyboard shortcuts", () => {
  it("accepts Ctrl shortcuts and keeps Command compatibility", () => {
    expect(
      resolveKeyboardCommand({
        key: ",",
        ctrlKey: true,
        metaKey: false,
        editing: false,
        phase: "idle",
        skipConfirmation: "none",
      }),
    ).toBe("show_main");
    expect(
      resolveKeyboardCommand({
        key: "P",
        ctrlKey: true,
        metaKey: false,
        editing: false,
        phase: "working",
        skipConfirmation: "none",
      }),
    ).toBe("toggle_pause");
    expect(
      resolveKeyboardCommand({
        key: "q",
        ctrlKey: true,
        metaKey: false,
        editing: false,
        phase: "idle",
        skipConfirmation: "none",
      }),
    ).toBe("quit");
    expect(
      resolveKeyboardCommand({
        key: ",",
        ctrlKey: false,
        metaKey: true,
        editing: false,
        phase: "idle",
        skipConfirmation: "none",
      }),
    ).toBe("show_main");
  });

  it("does not treat space in editable content as a timer command", () => {
    const input = document.createElement("input");
    const editable = document.createElement("div");
    editable.setAttribute("contenteditable", "true");

    expect(isEditingTarget(input)).toBe(true);
    expect(isEditingTarget(editable)).toBe(true);
    expect(
      resolveKeyboardCommand({
        key: " ",
        ctrlKey: false,
        metaKey: false,
        editing: true,
        phase: "working",
        skipConfirmation: "none",
      }),
    ).toBeNull();
  });

  it("only cancels an active skip confirmation with Escape", () => {
    expect(
      resolveKeyboardCommand({
        key: "Escape",
        ctrlKey: false,
        metaKey: false,
        editing: false,
        phase: "resting",
        skipConfirmation: "pending",
      }),
    ).toBe("cancel_skip");
  });

  it("does not treat AltGr or Ctrl+Alt text input as an app shortcut", () => {
    const base = {
      key: "q",
      ctrlKey: true,
      metaKey: false,
      editing: false,
      phase: "working" as const,
      skipConfirmation: "none" as const,
    };
    expect(
      resolveKeyboardCommand({ ...base, altKey: true, altGraph: false }),
    ).toBeNull();
    expect(
      resolveKeyboardCommand({ ...base, altKey: false, altGraph: true }),
    ).toBeNull();
  });
});
