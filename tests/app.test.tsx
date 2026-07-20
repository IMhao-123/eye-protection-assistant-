import { render, screen } from "@testing-library/react";
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
