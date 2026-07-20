import { Eye, Pause, Play, X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { appBridge } from "../bridge";
import { formatTime } from "../format";
import { getTranslation } from "../i18n";
import type { AppSnapshot, TimerAction } from "../types";

export function WidgetView({ snapshot, dispatch }: { snapshot: AppSnapshot; dispatch: (action: TimerAction) => Promise<void> }) {
  const t = getTranslation(snapshot.settings.language);
  const drag = async (event: React.MouseEvent) => {
    if ((event.target as HTMLElement).closest("button")) return;
    if ("__TAURI_INTERNALS__" in window) await getCurrentWindow().startDragging();
  };
  return (
    <main className="widget" onMouseDown={(event) => void drag(event)}>
      <div className="widget__brand"><Eye size={19} /></div>
      <button className="widget__time" onClick={() => void dispatch("toggle_pause")} aria-label={snapshot.phase === "paused" ? t.resume : t.pause}>
        <span>{t.timeLeft}</span><strong>{formatTime(snapshot.secondsRemaining)}</strong>
      </button>
      <button className="widget__control" onClick={() => void dispatch("toggle_pause")} aria-label={snapshot.phase === "paused" ? t.resume : t.pause}>{snapshot.phase === "paused" ? <Play size={15} fill="currentColor" /> : <Pause size={15} fill="currentColor" />}</button>
      <button className="widget__close" onClick={() => void appBridge.setWidgetVisibility(false)} aria-label={t.hideWidget}><X size={14} /></button>
    </main>
  );
}
