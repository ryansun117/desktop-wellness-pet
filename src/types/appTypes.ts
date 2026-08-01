export type ReminderKind = "water" | "stand";

export type ReminderState =
  | { status: "disabled" }
  | { status: "scheduled"; dueAt: string }
  | {
      status: "due";
      dueSince: string;
      eventEmitted: boolean;
      notificationEmitted: boolean;
    }
  | { status: "snoozed"; until: string };

export interface ReminderSnapshot {
  state: ReminderState;
  lastCompletedAt: string | null;
}

export interface AppSettings {
  waterEnabled: boolean;
  waterIntervalMinutes: number;
  standEnabled: boolean;
  standIntervalMinutes: number;
  snoozeDurationMinutes: number;
  quietHoursEnabled: boolean;
  quietHoursStart: string;
  quietHoursEnd: string;
  alwaysOnTop: boolean;
  launchAtLogin: boolean;
}

export interface AppSnapshot {
  settings: AppSettings;
  water: ReminderSnapshot;
  stand: ReminderSnapshot;
  currentReminder: ReminderKind | null;
  paused: boolean;
  pausedUntil: string | null;
}

export interface ValidationIssue {
  field: string;
  message: string;
}

export interface CommandError {
  code: string;
  details?: string | ValidationIssue[];
}

