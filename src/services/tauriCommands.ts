import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, AppSnapshot, CommandError, ReminderKind } from "../types/appTypes";

export const getAppSnapshot = () => invoke<AppSnapshot>("get_app_snapshot");
export const updateSettings = (settings: AppSettings) =>
  invoke<AppSnapshot>("update_settings", { settings });
export const pauseReminders = () =>
  invoke<AppSnapshot>("pause_reminders", { until: null });
export const resumeReminders = () => invoke<AppSnapshot>("resume_reminders");
export const resetSettings = () => invoke<AppSnapshot>("reset_settings");
export const openSettings = () => invoke<void>("open_settings");
export const closeSettings = () => invoke<void>("close_settings");

export function performReminderAction(
  kind: ReminderKind,
  action: "done" | "snooze" | "skip",
) {
  const command = {
    done: "complete_reminder",
    snooze: "snooze_reminder",
    skip: "skip_reminder",
  }[action];
  return invoke<AppSnapshot>(command, { kind });
}

export function describeError(reason: unknown): string {
  if (typeof reason === "string") return reason;
  if (reason && typeof reason === "object") {
    const error = reason as Partial<CommandError>;
    if (Array.isArray(error.details)) {
      return error.details.map((issue) => issue.message).join(" ");
    }
    if (typeof error.details === "string") return error.details;
    if (typeof error.code === "string") return error.code;
  }
  return "Something went wrong. Please try again.";
}

