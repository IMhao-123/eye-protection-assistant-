import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { disable, enable } from "@tauri-apps/plugin-autostart";
import type { AppSettings, AppSnapshot, TimerAction } from "./types";
import { DEFAULT_SNAPSHOT } from "./types";

type SnapshotListener = (snapshot: AppSnapshot) => void;

const isTauri = () => "__TAURI_INTERNALS__" in window;

class BrowserMock {
  private snapshot: AppSnapshot = structuredClone(DEFAULT_SNAPSHOT);
  private listeners = new Set<SnapshotListener>();
  private interval: number | null = null;

  getSnapshot() {
    return structuredClone(this.snapshot);
  }

  subscribe(listener: SnapshotListener) {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  dispatch(action: TimerAction) {
    const { settings } = this.snapshot;
    if (action === "start" && this.snapshot.phase === "idle") {
      this.snapshot = { ...this.snapshot, phase: "working", secondsRemaining: settings.workMinutes * 60 };
      this.startTicking();
    } else if (action === "toggle_pause" || action === "pause" || action === "resume") {
      if (this.snapshot.phase === "working") this.snapshot = { ...this.snapshot, phase: "paused" };
      else if (this.snapshot.phase === "paused") this.snapshot = { ...this.snapshot, phase: "working" };
    } else if (action === "stop") {
      this.snapshot = { ...this.snapshot, phase: "idle", secondsRemaining: 0, skipConfirmation: "none" };
      this.stopTicking();
    } else if (action === "request_skip" && this.snapshot.phase === "resting") {
      this.snapshot = settings.skipConfirmation
        ? { ...this.snapshot, skipConfirmation: "pending" }
        : { ...this.snapshot, phase: "working", secondsRemaining: settings.workMinutes * 60 };
    } else if (action === "cancel_skip") {
      this.snapshot = { ...this.snapshot, skipConfirmation: "none" };
    } else if (action === "confirm_skip" && this.snapshot.skipConfirmation === "pending") {
      this.snapshot = {
        ...this.snapshot,
        phase: "working",
        secondsRemaining: settings.workMinutes * 60,
        skipConfirmation: "none",
      };
    }
    this.emit();
    return this.getSnapshot();
  }

  updateSettings(settings: AppSettings) {
    this.snapshot = { ...this.snapshot, settings };
    this.emit();
    return this.getSnapshot();
  }

  setWidgetVisibility(visible: boolean) {
    return this.updateSettings({ ...this.snapshot.settings, widgetVisible: visible });
  }

  reset(snapshot: AppSnapshot = DEFAULT_SNAPSHOT) {
    this.stopTicking();
    this.snapshot = structuredClone(snapshot);
    this.emit();
  }

  private startTicking() {
    if (this.interval !== null) return;
    this.interval = window.setInterval(() => {
      if (this.snapshot.phase !== "working" && this.snapshot.phase !== "resting") return;
      const remaining = Math.max(0, this.snapshot.secondsRemaining - 1);
      if (remaining === 0) {
        if (this.snapshot.phase === "working") {
          this.snapshot = { ...this.snapshot, phase: "resting", secondsRemaining: this.snapshot.settings.restSeconds };
        } else {
          this.snapshot = {
            ...this.snapshot,
            phase: "working",
            secondsRemaining: this.snapshot.settings.workMinutes * 60,
          };
        }
      } else {
        this.snapshot = { ...this.snapshot, secondsRemaining: remaining };
      }
      this.emit();
    }, 1000);
  }

  private stopTicking() {
    if (this.interval !== null) window.clearInterval(this.interval);
    this.interval = null;
  }

  private emit() {
    const snapshot = this.getSnapshot();
    this.listeners.forEach((listener) => listener(snapshot));
  }
}

export const browserMock = new BrowserMock();

export const appBridge = {
  async getSnapshot(): Promise<AppSnapshot> {
    return isTauri() ? invoke("get_app_snapshot") : browserMock.getSnapshot();
  },
  async dispatch(action: TimerAction): Promise<AppSnapshot> {
    return isTauri()
      ? invoke("dispatch_timer_action", { action: { type: action } })
      : browserMock.dispatch(action);
  },
  async updateSettings(settings: AppSettings): Promise<AppSnapshot> {
    if (!isTauri()) return browserMock.updateSettings(settings);
    const snapshot = await invoke<AppSnapshot>("update_settings", { settings });
    try {
      if (settings.launchAtLogin) await enable();
      else await disable();
    } catch (error) {
      console.warn("Unable to update login launch preference", error);
    }
    return snapshot;
  },
  async setWidgetVisibility(visible: boolean): Promise<AppSnapshot> {
    return isTauri()
      ? invoke("set_widget_visibility", { visible })
      : browserMock.setWidgetVisibility(visible);
  },
  async showMainWindow() {
    if (isTauri()) await invoke("show_main_window");
  },
  async quit() {
    if (isTauri()) await invoke("quit_app");
  },
  async subscribe(listener: SnapshotListener): Promise<UnlistenFn> {
    if (!isTauri()) return browserMock.subscribe(listener);
    return listen<AppSnapshot>("app://snapshot-changed", (event) => listener(event.payload));
  },
};
