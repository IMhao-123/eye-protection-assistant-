import { useEffect, useState } from "react";
import {
  Bell,
  CirclePlay,
  Eye,
  MoonStar,
  Pause,
  Play,
  Settings2,
  SlidersHorizontal,
  Speaker,
  Square,
} from "lucide-react";
import { formatTime } from "../format";
import { getTranslation } from "../i18n";
import type { AppSettings, AppSnapshot, TimerAction } from "../types";

type Section = "overview" | "reminder" | "break" | "sound" | "appearance" | "system";

interface MainViewProps {
  snapshot: AppSnapshot;
  dispatch: (action: TimerAction) => Promise<void>;
  updateSettings: (settings: AppSettings) => Promise<void>;
}

export function MainView({ snapshot, dispatch, updateSettings }: MainViewProps) {
  const [section, setSection] = useState<Section>("overview");
  const t = getTranslation(snapshot.settings.language);
  const phaseLabel = {
    idle: t.ready,
    working: t.working,
    paused: t.paused,
    resting: t.resting,
  }[snapshot.phase];

  const nav = [
    ["overview", t.overview, CirclePlay],
    ["reminder", t.reminder, Bell],
    ["break", t.breakExperience, Eye],
    ["sound", t.sound, Speaker],
    ["appearance", t.appearance, MoonStar],
    ["system", t.system, Settings2],
  ] as const;

  return (
    <main className="app-shell">
      <aside className="sidebar" aria-label={t.settings}>
        <div className="brand">
          <div className="brand__icon"><Eye size={23} strokeWidth={1.8} /></div>
          <div><strong>{t.appName}</strong><span>{t.tagline}</span></div>
        </div>
        <nav className="nav-list">
          {nav.map(([id, label, Icon]) => (
            <button
              key={id}
              className={section === id ? "nav-item is-active" : "nav-item"}
              onClick={() => setSection(id)}
              aria-current={section === id ? "page" : undefined}
            >
              <Icon size={17} /> <span>{label}</span>
            </button>
          ))}
        </nav>
        <div className="sidebar__foot"><span className="status-dot" />{phaseLabel}</div>
      </aside>

      <section className="content">
        {snapshot.recoverableError && (
          <div className="notice" role="status">{t.recovered}</div>
        )}
        {section === "overview" && (
          <Overview snapshot={snapshot} dispatch={dispatch} t={t} />
        )}
        {section === "reminder" && (
          <ReminderSettings settings={snapshot.settings} update={updateSettings} t={t} />
        )}
        {section === "break" && (
          <BreakSettings settings={snapshot.settings} update={updateSettings} t={t} />
        )}
        {section === "sound" && (
          <SoundSettings settings={snapshot.settings} update={updateSettings} t={t} />
        )}
        {section === "appearance" && (
          <AppearanceSettings settings={snapshot.settings} update={updateSettings} t={t} />
        )}
        {section === "system" && (
          <SystemSettings settings={snapshot.settings} update={updateSettings} t={t} />
        )}
      </section>
    </main>
  );
}

type T = ReturnType<typeof getTranslation>;

function Overview({ snapshot, dispatch, t }: { snapshot: AppSnapshot; dispatch: (action: TimerAction) => Promise<void>; t: T }) {
  const active = snapshot.phase !== "idle";
  return (
    <div className="page overview-page">
      <header className="page-header">
        <span className="eyebrow">20–20–20</span>
        <h1>{active ? (snapshot.phase === "paused" ? t.paused : t.working) : t.ready}</h1>
        <p>{t.cycleSummary(snapshot.settings.workMinutes, snapshot.settings.restSeconds)}</p>
      </header>

      <div className="focus-orbit" aria-label={active ? formatTime(snapshot.secondsRemaining) : t.ready}>
        <div className="focus-orbit__horizon" />
        <div className="focus-orbit__center">
          {active ? <strong>{formatTime(snapshot.secondsRemaining)}</strong> : <Eye size={50} strokeWidth={1.25} />}
          <span>{active ? (snapshot.phase === "paused" ? t.paused : t.working) : "20 · 20 · 20"}</span>
        </div>
      </div>

      <div className="overview-actions">
        {!active ? (
          <button className="button button--primary button--large" onClick={() => void dispatch("start")}>
            <Play size={18} fill="currentColor" />{t.start}
          </button>
        ) : (
          <>
            <button className="button button--primary" onClick={() => void dispatch("toggle_pause")}>
              {snapshot.phase === "paused" ? <Play size={17} /> : <Pause size={17} />}
              {snapshot.phase === "paused" ? t.resume : t.pause}
            </button>
            <button className="button button--secondary" onClick={() => void dispatch("stop")}>
              <Square size={15} fill="currentColor" />{t.stop}
            </button>
          </>
        )}
      </div>
    </div>
  );
}

function SectionHeader({ title, description }: { title: string; description: string }) {
  return <header className="page-header compact"><h1>{title}</h1><p>{description}</p></header>;
}

function ReminderSettings({ settings, update, t }: SettingsProps) {
  return (
    <div className="page settings-page">
      <SectionHeader title={t.reminder} description={t.cycleSummary(settings.workMinutes, settings.restSeconds)} />
      <div className="settings-grid">
        <NumberField label={t.workDuration} unit={t.minutes} value={settings.workMinutes} min={1} max={120} onChange={(value) => update({ ...settings, workMinutes: value })} />
        <NumberField label={t.restDuration} unit={t.seconds} value={settings.restSeconds} min={5} max={300} onChange={(value) => update({ ...settings, restSeconds: value })} />
      </div>
      <div className="setting-row"><div><strong>{t.notificationEnabled}</strong><p>在阶段切换时显示 macOS 通知</p></div><Switch checked={settings.notificationEnabled} label={t.notificationEnabled} onChange={(value) => update({ ...settings, notificationEnabled: value })} /></div>
      <AutoSave t={t} />
    </div>
  );
}

function BreakSettings({ settings, update, t }: SettingsProps) {
  return (
    <div className="page settings-page">
      <SectionHeader title={t.breakExperience} description="覆盖全部显示器，帮助你真正离开近距离画面。" />
      <label className="field field--full"><span>{t.breakMessage}</span><textarea rows={3} maxLength={120} value={settings.breakMessage} onChange={(event) => void update({ ...settings, breakMessage: event.target.value })} /></label>
      <div className="setting-row"><div><strong>{t.confirmation}</strong><p>防止无意中提前结束短暂休息</p></div><Switch checked={settings.skipConfirmation} label={t.confirmation} onChange={(value) => update({ ...settings, skipConfirmation: value })} /></div>
      <label className="field field--full"><span>{t.sleepPolicy}</span><select value={settings.sleepPolicy} onChange={(event) => void update({ ...settings, sleepPolicy: event.target.value as AppSettings["sleepPolicy"] })}><option value="restart_cycle">{t.restartCycle}</option><option value="pause_resume">{t.pauseResume}</option><option value="real_time">{t.realTime}</option></select></label>
      <AutoSave t={t} />
    </div>
  );
}

function SoundSettings({ settings, update, t }: SettingsProps) {
  return <div className="page settings-page"><SectionHeader title={t.sound} description="提示音由应用本地生成，断网时仍然可用。" /><div className="setting-row"><div><strong>{t.soundEnabled}</strong><p>仅在工作和休息自动切换时播放</p></div><Switch checked={settings.soundEnabled} label={t.soundEnabled} onChange={(value) => update({ ...settings, soundEnabled: value })} /></div><div className="sound-preview" aria-hidden="true"><span /><span /><span /><span /></div><AutoSave t={t} /></div>;
}

function AppearanceSettings({ settings, update, t }: SettingsProps) {
  return <div className="page settings-page"><SectionHeader title={t.appearance} description="使用系统字体和柔和自然色，自动适配减少动态效果。" /><label className="field field--full"><span>{t.theme}</span><div className="segmented">{(["system", "light", "dark"] as const).map((value) => <button key={value} className={settings.theme === value ? "is-selected" : ""} onClick={() => void update({ ...settings, theme: value })}>{value === "system" ? t.followSystem : value === "light" ? t.light : t.dark}</button>)}</div></label><label className="field field--full"><span>{t.language}</span><div className="segmented">{(["system", "zh", "en"] as const).map((value) => <button key={value} className={settings.language === value ? "is-selected" : ""} onClick={() => void update({ ...settings, language: value })}>{value === "system" ? t.followSystem : value === "zh" ? t.chinese : t.english}</button>)}</div></label><AutoSave t={t} /></div>;
}

function SystemSettings({ settings, update, t }: SettingsProps) {
  return <div className="page settings-page"><SectionHeader title={t.system} description="护眼助手关闭主窗口后仍在菜单栏继续计时。" /><div className="setting-row"><div><strong>{t.launchAtLogin}</strong><p>默认关闭，可随时从这里更改</p></div><Switch checked={settings.launchAtLogin} label={t.launchAtLogin} onChange={(value) => update({ ...settings, launchAtLogin: value })} /></div><div className="setting-row"><div><strong>{t.widgetVisible}</strong><p>胶囊可拖动，并会记住最后位置</p></div><Switch checked={settings.widgetVisible} label={t.widgetVisible} onChange={(value) => update({ ...settings, widgetVisible: value })} /></div><AutoSave t={t} /></div>;
}

interface SettingsProps { settings: AppSettings; update: (settings: AppSettings) => Promise<void>; t: T }

function NumberField({ label, unit, value, min, max, onChange }: { label: string; unit: string; value: number; min: number; max: number; onChange: (value: number) => Promise<void> }) {
  const [draft, setDraft] = useState(String(value));
  useEffect(() => setDraft(String(value)), [value]);
  const commit = () => {
    const next = Math.max(min, Math.min(max, Number(draft) || min));
    setDraft(String(next));
    void onChange(next);
  };
  return <label className="field"><span>{label}</span><div className="number-input"><input aria-label={label} type="number" value={draft} min={min} max={max} onChange={(event) => setDraft(event.target.value)} onBlur={commit} onKeyDown={(event) => { if (event.key === "Enter") commit(); }} /><small>{unit}</small></div><em>{min}–{max} {unit}</em></label>;
}

function Switch({ checked, onChange, label }: { checked: boolean; onChange: (value: boolean) => Promise<void>; label: string }) {
  return <button role="switch" aria-checked={checked} aria-label={label} className={checked ? "switch is-on" : "switch"} onClick={() => void onChange(!checked)}><span /></button>;
}

function AutoSave({ t }: { t: T }) {
  return <p className="autosave"><SlidersHorizontal size={14} />{t.savedAutomatically}</p>;
}
