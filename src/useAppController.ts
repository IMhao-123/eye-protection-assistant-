import { useCallback, useEffect, useRef, useState } from "react";
import { appBridge } from "./bridge";
import type { AppSettings, AppSnapshot, TimerAction } from "./types";
import { DEFAULT_SNAPSHOT } from "./types";

export function useAppController() {
  const [snapshot, setSnapshot] = useState<AppSnapshot>(DEFAULT_SNAPSHOT);
  const [loading, setLoading] = useState(true);
  const previousPhase = useRef(snapshot.phase);

  useEffect(() => {
    let active = true;
    let unsubscribe: (() => void) | undefined;
    void appBridge.getSnapshot().then((next) => {
      if (active) {
        setSnapshot(next);
        setLoading(false);
      }
    });
    void appBridge.subscribe((next) => active && setSnapshot(next)).then((fn) => {
      unsubscribe = fn;
    });
    return () => {
      active = false;
      unsubscribe?.();
    };
  }, []);

  useEffect(() => {
    if (
      snapshot.settings.soundEnabled &&
      previousPhase.current !== snapshot.phase &&
      (snapshot.phase === "resting" || previousPhase.current === "resting") &&
      new URLSearchParams(window.location.search).get("view") !== "break"
    ) {
      playLocalChime();
    }
    previousPhase.current = snapshot.phase;
  }, [snapshot.phase, snapshot.settings.soundEnabled]);

  const dispatch = useCallback(async (action: TimerAction) => {
    setSnapshot(await appBridge.dispatch(action));
  }, []);

  const updateSettings = useCallback(async (settings: AppSettings) => {
    setSnapshot(await appBridge.updateSettings(settings));
  }, []);

  return { snapshot, loading, dispatch, updateSettings };
}

function playLocalChime() {
  const AudioContextClass = window.AudioContext;
  if (!AudioContextClass) return;
  const context = new AudioContextClass();
  const gain = context.createGain();
  gain.gain.setValueAtTime(0.0001, context.currentTime);
  gain.gain.exponentialRampToValueAtTime(0.12, context.currentTime + 0.02);
  gain.gain.exponentialRampToValueAtTime(0.0001, context.currentTime + 0.75);
  gain.connect(context.destination);
  [523.25, 659.25].forEach((frequency, index) => {
    const oscillator = context.createOscillator();
    oscillator.type = "sine";
    oscillator.frequency.value = frequency;
    oscillator.connect(gain);
    oscillator.start(context.currentTime + index * 0.08);
    oscillator.stop(context.currentTime + 0.8);
  });
  window.setTimeout(() => void context.close(), 900);
}
