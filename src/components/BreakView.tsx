import { Eye, X } from "lucide-react";
import { formatTime, progressValue } from "../format";
import { getTranslation } from "../i18n";
import type { AppSnapshot, TimerAction } from "../types";

export function BreakView({ snapshot, dispatch }: { snapshot: AppSnapshot; dispatch: (action: TimerAction) => Promise<void> }) {
  const t = getTranslation(snapshot.settings.language);
  const total = snapshot.settings.restSeconds;
  const progress = progressValue(snapshot.secondsRemaining, total);
  return (
    <main className="break-view">
      <div className="break-view__glow break-view__glow--one" />
      <div className="break-view__glow break-view__glow--two" />
      <div className="break-content">
        <div className="break-mark"><Eye size={28} strokeWidth={1.5} /></div>
        <p className="break-kicker">20–20–20</p>
        <h1>{t.breakTitle}</h1>
        <p className="break-message">{snapshot.settings.breakMessage || t.breakFallback}</p>
        <strong className="break-time">{formatTime(snapshot.secondsRemaining)}</strong>
        <div className="break-progress" role="progressbar" aria-valuemin={0} aria-valuemax={total} aria-valuenow={snapshot.secondsRemaining}><span style={{ transform: `scaleX(${progress})` }} /></div>
        <button className="break-skip" onClick={() => void dispatch("request_skip")}>{t.skipBreak}</button>
      </div>

      {snapshot.skipConfirmation === "pending" && (
        <div className="modal-backdrop" role="presentation">
          <section className="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="skip-title">
            <button className="dialog-close" onClick={() => void dispatch("cancel_skip")} aria-label={t.close}><X size={18} /></button>
            <div className="confirm-dialog__mark"><Eye size={27} /></div>
            <h2 id="skip-title">{t.skipTitle}</h2>
            <p>{t.skipDescription}</p>
            <div className="dialog-actions"><button className="button button--primary" autoFocus onClick={() => void dispatch("cancel_skip")}>{t.keepResting}</button><button className="button button--quiet" onClick={() => void dispatch("confirm_skip")}>{t.confirmSkip}</button></div>
          </section>
        </div>
      )}
    </main>
  );
}
