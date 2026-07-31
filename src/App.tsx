import { useEffect } from "react";
import { appBridge } from "./bridge";
import { BreakView } from "./components/BreakView";
import { MainView } from "./components/MainView";
import { WidgetView } from "./components/WidgetView";
import {
  isEditingTarget,
  resolveKeyboardCommand,
} from "./keyboardShortcuts";
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
      const command = resolveKeyboardCommand({
        key: event.key,
        ctrlKey: event.ctrlKey,
        metaKey: event.metaKey,
        altKey: event.altKey,
        altGraph: event.getModifierState("AltGraph"),
        editing: isEditingTarget(event.target),
        phase: snapshot.phase,
        skipConfirmation: snapshot.skipConfirmation,
      });
      if (!command) return;

      event.preventDefault();
      switch (command) {
        case "show_main":
          void appBridge.showMainWindow();
          break;
        case "toggle_pause":
          void dispatch("toggle_pause");
          break;
        case "cancel_skip":
          void dispatch("cancel_skip");
          break;
        case "quit":
          void appBridge.quit();
          break;
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
