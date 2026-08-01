import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AppSnapshot, ReminderKind } from "../types/appTypes";

interface ReminderDueEvent {
  kind: ReminderKind;
  snapshot: AppSnapshot;
}

export async function listenForSnapshots(onSnapshot: (snapshot: AppSnapshot) => void) {
  const listeners = await Promise.all([
    listen<ReminderDueEvent>("reminder-due", ({ payload }) => onSnapshot(payload.snapshot)),
    listen<AppSnapshot>("reminder-updated", ({ payload }) => onSnapshot(payload)),
    listen<AppSnapshot>("settings-updated", ({ payload }) => onSnapshot(payload)),
    listen<AppSnapshot>("pause-state-changed", ({ payload }) => onSnapshot(payload)),
  ]);
  return () => listeners.forEach((unlisten: UnlistenFn) => unlisten());
}

