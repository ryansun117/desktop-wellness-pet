import type { AppSnapshot } from "../types/appTypes";

export const snapshot: AppSnapshot = {
  settings: {
    waterEnabled: true,
    waterIntervalMinutes: 45,
    standEnabled: true,
    standIntervalMinutes: 60,
    snoozeDurationMinutes: 10,
    quietHoursEnabled: true,
    quietHoursStart: "22:00",
    quietHoursEnd: "08:00",
    alwaysOnTop: true,
    launchAtLogin: false,
  },
  water: {
    state: { status: "due", dueSince: "2026-08-01T12:00:00Z", eventEmitted: true, notificationEmitted: true },
    lastCompletedAt: null,
  },
  stand: {
    state: { status: "scheduled", dueAt: "2026-08-01T13:00:00Z" },
    lastCompletedAt: null,
  },
  currentReminder: "water",
  paused: false,
  pausedUntil: null,
};

