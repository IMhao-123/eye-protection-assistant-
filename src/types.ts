export type TimerPhase = "idle" | "working" | "paused" | "resting";
export type SleepPolicy = "restart_cycle" | "pause_resume" | "real_time";
export type AppLanguage = "system" | "zh" | "en";
export type ThemePreference = "system" | "light" | "dark";
export type ColorScheme =
  | "original"
  | "morning_lake"
  | "graphite_lime"
  | "mist_blue_coral"
  | "porcelain_forest";
export type SkipConfirmationState = "none" | "pending";

export const AVAILABLE_COLOR_SCHEMES = [
  "original",
  "morning_lake",
  "graphite_lime",
  "mist_blue_coral",
  "porcelain_forest",
] as const satisfies readonly ColorScheme[];

export interface AppSettings {
  version: number;
  workMinutes: number;
  restSeconds: number;
  skipConfirmation: boolean;
  sleepPolicy: SleepPolicy;
  language: AppLanguage;
  theme: ThemePreference;
  colorScheme: ColorScheme;
  soundEnabled: boolean;
  notificationEnabled: boolean;
  launchAtLogin: boolean;
  widgetVisible: boolean;
  breakMessage: string;
}

export interface RecoverableAppError {
  code: string;
  message: string;
}

export interface AppSnapshot {
  phase: TimerPhase;
  secondsRemaining: number;
  skipConfirmation: SkipConfirmationState;
  settings: AppSettings;
  recoverableError: RecoverableAppError | null;
}

export type TimerAction =
  | "start"
  | "pause"
  | "resume"
  | "toggle_pause"
  | "stop"
  | "request_skip"
  | "confirm_skip"
  | "cancel_skip"
  | "sleep"
  | "wake"
  | "tick";

export const DEFAULT_SETTINGS: AppSettings = {
  version: 1,
  workMinutes: 20,
  restSeconds: 20,
  skipConfirmation: true,
  sleepPolicy: "restart_cycle",
  language: "system",
  theme: "system",
  colorScheme: "mist_blue_coral",
  soundEnabled: true,
  notificationEnabled: true,
  launchAtLogin: false,
  widgetVisible: true,
  breakMessage: "请眺望远方，让眼睛放松。",
};

export const DEFAULT_SNAPSHOT: AppSnapshot = {
  phase: "idle",
  secondsRemaining: 0,
  skipConfirmation: "none",
  settings: DEFAULT_SETTINGS,
  recoverableError: null,
};
