import { useEffect, useMemo, useState } from "react";
import "./App.css";
import { Pet, type PetMood } from "./components/Pet/Pet";
import { ReminderBubble } from "./components/ReminderBubble/ReminderBubble";
import { SettingsPanel } from "./components/SettingsPanel/SettingsPanel";
import { useAppSnapshot } from "./hooks/useAppSnapshot";
import * as commands from "./services/tauriCommands";
import type { ReminderKind } from "./types/appTypes";

export default function App() {
  const { snapshot, loading, error, setError, replaceSnapshot } = useAppSnapshot();
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [celebrating, setCelebrating] = useState(false);

  useEffect(() => {
    if (!celebrating) return;
    const timer = window.setTimeout(() => setCelebrating(false), 1400);
    return () => window.clearTimeout(timer);
  }, [celebrating]);

  const mood = useMemo<PetMood>(() => {
    if (celebrating) return "celebrating";
    if (snapshot?.paused) return "paused";
    if (snapshot?.currentReminder === "water") return "waterDue";
    if (snapshot?.currentReminder === "stand") return "standDue";
    if (
      snapshot?.water.state.status === "snoozed" ||
      snapshot?.stand.state.status === "snoozed"
    ) {
      return "snoozed";
    }
    return "idle";
  }, [celebrating, snapshot]);

  async function openSettings() {
    try {
      await commands.openSettings();
      setSettingsOpen(true);
    } catch (reason) {
      setError(commands.describeError(reason));
    }
  }

  async function closeSettings() {
    try {
      await commands.closeSettings();
      setSettingsOpen(false);
    } catch (reason) {
      setError(commands.describeError(reason));
    }
  }

  async function act(kind: ReminderKind, action: "done" | "snooze" | "skip") {
    try {
      const next = await commands.performReminderAction(kind, action);
      replaceSnapshot(next);
      if (action === "done") setCelebrating(true);
    } catch (reason) {
      setError(commands.describeError(reason));
    }
  }

  return (
    <main className={`app ${settingsOpen ? "settings-open" : "compact"}`}>
      <div className="drag-handle" data-tauri-drag-region aria-label="Drag Wellness Pet">
        <span data-tauri-drag-region />
        <span data-tauri-drag-region />
        <span data-tauri-drag-region />
      </div>

      {error && (
        <div className="error-banner" role="alert">
          <span>{error}</span>
          <button type="button" aria-label="Dismiss error" onClick={() => setError(null)}>
            ×
          </button>
        </div>
      )}

      {loading && !snapshot ? (
        <div className="loading" role="status">Waking up…</div>
      ) : (
        <>
          <ReminderBubble
            kind={snapshot?.currentReminder ?? null}
            onDone={(kind) => void act(kind, "done")}
            onSnooze={(kind) => void act(kind, "snooze")}
            onSkip={(kind) => void act(kind, "skip")}
          />
          <Pet mood={mood} onOpenSettings={() => void openSettings()} />
          {!settingsOpen && (
            <button className="settings-button" type="button" onClick={() => void openSettings()}>
              Settings
            </button>
          )}
        </>
      )}

      {settingsOpen && snapshot && (
        <SettingsPanel
          key={JSON.stringify(snapshot.settings)}
          snapshot={snapshot}
          onSnapshot={replaceSnapshot}
          onError={setError}
          onClose={() => void closeSettings()}
        />
      )}
    </main>
  );
}
