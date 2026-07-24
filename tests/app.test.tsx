import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import App from "../src/App";
import { browserMock } from "../src/bridge";
import { DEFAULT_SETTINGS, DEFAULT_SNAPSHOT } from "../src/types";

const zhSnapshot = {
  ...DEFAULT_SNAPSHOT,
  settings: { ...DEFAULT_SETTINGS, language: "zh" as const },
};

describe("main application", () => {
  beforeEach(() => browserMock.reset(zhSnapshot));

  it("starts a focus session from the overview", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(await screen.findByRole("button", { name: "开始专注" }));
    expect(await screen.findByRole("heading", { name: "正在专注" })).toBeInTheDocument();
    expect(screen.getByLabelText(/^19:5\d$|^20:00$/)).toBeInTheDocument();
  });

  it("updates bounded reminder durations", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(await screen.findByRole("button", { name: "提醒" }));
    const work = screen.getByRole("spinbutton", { name: "工作时长" });
    await user.clear(work);
    await user.type(work, "42");
    expect(work).toHaveValue(42);
  });

  it("shows a recovery notice without blocking the app", async () => {
    browserMock.reset({
      ...zhSnapshot,
      recoverableError: { code: "settings_recovered", message: "broken" },
    });
    render(<App />);
    expect(await screen.findByRole("status")).toHaveTextContent("设置已安全恢复");
    expect(screen.getByRole("button", { name: "开始专注" })).toBeEnabled();
  });

  it("applies the saved color scheme to the document surface", async () => {
    browserMock.reset({
      ...zhSnapshot,
      settings: { ...zhSnapshot.settings, colorScheme: "morning_lake" },
    });
    render(<App />);
    await screen.findByRole("button", { name: "开始专注" });
    expect(document.documentElement).toHaveAttribute("data-color-scheme", "morning_lake");
  });

  it("switches between the original and morning lake palettes", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(await screen.findByRole("button", { name: "外观" }));

    expect(screen.getByRole("button", { name: "雾蓝珊瑚" })).toHaveAttribute("aria-pressed", "true");
    await user.click(screen.getByRole("button", { name: "原配色" }));
    expect(browserMock.getSnapshot().settings.colorScheme).toBe("original");
    await user.click(screen.getByRole("button", { name: "清晨湖光" }));

    expect(document.documentElement).toHaveAttribute("data-color-scheme", "morning_lake");
    expect(browserMock.getSnapshot().settings.colorScheme).toBe("morning_lake");
    expect(screen.getByRole("button", { name: "清晨湖光" })).toHaveAttribute("aria-pressed", "true");
  });

  it("switches to graphite lime without changing the active timer", async () => {
    browserMock.reset({
      ...zhSnapshot,
      phase: "working",
      secondsRemaining: 873,
    });
    const user = userEvent.setup();
    render(<App />);
    await user.click(await screen.findByRole("button", { name: "外观" }));
    await user.click(screen.getByRole("button", { name: "石墨青柠" }));

    expect(document.documentElement).toHaveAttribute("data-color-scheme", "graphite_lime");
    expect(browserMock.getSnapshot()).toMatchObject({
      phase: "working",
      secondsRemaining: 873,
      settings: { colorScheme: "graphite_lime" },
    });
  });

  it("offers mist blue coral and applies it as the default palette", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(await screen.findByRole("button", { name: "外观" }));

    expect(screen.getByRole("button", { name: "雾蓝珊瑚" })).toHaveAttribute("aria-pressed", "true");
    expect(document.documentElement).toHaveAttribute("data-color-scheme", "mist_blue_coral");
  });

  it("switches through all five palettes with one selected option", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(await screen.findByRole("button", { name: "外观" }));
    const palettePicker = screen.getByRole("group", { name: "配色方案" });

    for (const [label, value] of [
      ["原配色", "original"],
      ["清晨湖光", "morning_lake"],
      ["石墨青柠", "graphite_lime"],
      ["雾蓝珊瑚", "mist_blue_coral"],
      ["白瓷森林", "porcelain_forest"],
    ] as const) {
      await user.click(screen.getByRole("button", { name: label }));
      expect(document.documentElement).toHaveAttribute("data-color-scheme", value);
      expect(browserMock.getSnapshot().settings.colorScheme).toBe(value);
      expect(within(palettePicker).getAllByRole("button", { pressed: true })).toHaveLength(1);
    }
  });
});

describe("break overlay", () => {
  beforeEach(() => {
    window.history.replaceState({}, "", "/?view=break");
    browserMock.reset({
      ...zhSnapshot,
      phase: "resting",
      secondsRemaining: 20,
    });
  });

  it("requires and can cancel skip confirmation", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(await screen.findByRole("button", { name: "提前结束休息" }));
    expect(screen.getByRole("dialog", { name: "确定现在结束休息吗？" })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "继续休息" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("continues working after confirming skip", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(await screen.findByRole("button", { name: "提前结束休息" }));
    await user.click(screen.getByRole("button", { name: "确认结束" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });
});
