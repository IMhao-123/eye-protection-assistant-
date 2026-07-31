import type { SkipConfirmationState, TimerPhase } from "./types";

export type KeyboardCommand =
  | "show_main"
  | "toggle_pause"
  | "cancel_skip"
  | "quit";

interface KeyboardCommandInput {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey?: boolean;
  altGraph?: boolean;
  editing: boolean;
  phase: TimerPhase;
  skipConfirmation: SkipConfirmationState;
}

export function isEditingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const contentEditable = target.getAttribute("contenteditable");
  return (
    target.matches("input, textarea, select") ||
    target.isContentEditable ||
    (contentEditable !== null && contentEditable !== "false") ||
    target.closest('[contenteditable="true"]') !== null
  );
}

export function resolveKeyboardCommand({
  key,
  ctrlKey,
  metaKey,
  altKey = false,
  altGraph = false,
  editing,
  phase,
  skipConfirmation,
}: KeyboardCommandInput): KeyboardCommand | null {
  const commandModifier = (ctrlKey || metaKey) && !altKey && !altGraph;
  const normalizedKey = key.toLowerCase();

  if (commandModifier && key === ",") return "show_main";
  if (
    !editing &&
    (key === " " || (commandModifier && normalizedKey === "p")) &&
    (phase === "working" || phase === "paused")
  ) {
    return "toggle_pause";
  }
  if (key === "Escape" && skipConfirmation === "pending") {
    return "cancel_skip";
  }
  if (commandModifier && normalizedKey === "q") return "quit";
  return null;
}
