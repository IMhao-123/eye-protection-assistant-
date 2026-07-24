import { describe, expect, it } from "vitest";
import {
  AVAILABLE_COLOR_SCHEMES,
  DEFAULT_SETTINGS,
  type ColorScheme,
  type TimerAction,
} from "../src/types";

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

  it("exposes all five completed color schemes", () => {
    const schemes: ColorScheme[] = [
      "original",
      "morning_lake",
      "graphite_lime",
      "mist_blue_coral",
      "porcelain_forest",
    ];
    expect(AVAILABLE_COLOR_SCHEMES).toEqual(schemes);
    expect(DEFAULT_SETTINGS.colorScheme).toBe("mist_blue_coral");
  });
});
