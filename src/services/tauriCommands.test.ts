import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { performReminderAction } from "./tauriCommands";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("typed reminder commands", () => {
  beforeEach(() => vi.mocked(invoke).mockReset());

  it.each([
    ["done", "complete_reminder"],
    ["snooze", "snooze_reminder"],
    ["skip", "skip_reminder"],
  ] as const)("sends %s through Rust", async (action, command) => {
    vi.mocked(invoke).mockResolvedValue({});
    await performReminderAction("stand", action);
    expect(invoke).toHaveBeenCalledWith(command, { kind: "stand" });
  });
});

