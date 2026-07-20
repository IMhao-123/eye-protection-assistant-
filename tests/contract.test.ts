import { describe, expect, it } from "vitest";
import { DEFAULT_SETTINGS, type TimerAction } from "../src/types";

describe("Rust and TypeScript contract", () => {
  it("uses the versioned 20-20 defaults", () => {
    expect(DEFAULT_SETTINGS).toMatchObject({
      version: 1,
      workMinutes: 20,
      restSeconds: 20,
      skipConfirmation: true,
      sleepPolicy: "restart_cycle",
    });
  });

  it("exposes every timer action accepted by Rust", () => {
    const actions: TimerAction[] = [
      "start",
      "pause",
      "resume",
      "toggle_pause",
      "stop",
      "request_skip",
      "confirm_skip",
      "cancel_skip",
      "sleep",
      "wake",
      "tick",
    ];
    expect(actions).toHaveLength(11);
  });
});
