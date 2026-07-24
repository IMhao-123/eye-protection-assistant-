import { useEffect } from "react";
import { appBridge } from "./bridge";
import { BreakView } from "./components/BreakView";
import { MainView } from "./components/MainView";
import { WidgetView } from "./components/WidgetView";
import { useAppController } from "./useAppController";
import { resolveViewMode } from "./viewSurface";

export default function App() {
  const controller = useAppController();
  const { snapshot, dispatch } = controller;
  const theme = snapshot.settings.theme;
  const colorScheme = snapshot.settings.colorScheme;
  const view = resolveViewMode(window.location.search);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    document.documentElement.dataset.colorScheme = colorScheme;
  }, [colorScheme, theme]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const editing = (event.target as HTMLElement | null)?.matches("input, textarea, select");
      if (event.metaKey && event.key === ",") {
        event.preventDefault();
        void appBridge.showMainWindow();
      } else if (
        !editing &&
        (event.key === " " || (event.metaKey && event.key.toLowerCase() === "p")) &&
        (snapshot.phase === "working" || snapshot.phase === "paused")
      ) {
        event.preventDefault();
        void dispatch("toggle_pause");
      } else if (event.key === "Escape" && snapshot.skipConfirmation === "pending") {
        event.preventDefault();
        void dispatch("cancel_skip");
      } else if (event.metaKey && event.key.toLowerCase() === "q") {
        event.preventDefault();
        void appBridge.quit();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [dispatch, snapshot.phase, snapshot.skipConfirmation]);

  if (controller.loading) return <div className="loading" aria-label="Loading" />;
  if (view === "widget") return <WidgetView {...controller} />;
  if (view === "break") return <BreakView {...controller} />;
  return <MainView {...controller} />;
}
