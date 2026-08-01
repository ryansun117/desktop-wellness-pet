import { act, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { snapshot } from "../test/fixtures";
import type { AppSnapshot } from "../types/appTypes";
import { useAppSnapshot } from "./useAppSnapshot";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));

type Listener = (event: { payload: unknown }) => void;

function Harness() {
  const state = useAppSnapshot();
  return <div>{state.loading ? "loading" : state.snapshot?.currentReminder ?? "none"}</div>;
}

describe("useAppSnapshot", () => {
  const callbacks = new Map<string, Listener>();
  const unlisten = vi.fn();

  beforeEach(() => {
    callbacks.clear();
    unlisten.mockReset();
    vi.mocked(invoke).mockReset();
    vi.mocked(listen).mockReset();
    vi.mocked(invoke).mockResolvedValue(snapshot);
    vi.mocked(listen).mockImplementation(async (event, handler) => {
      callbacks.set(event, handler as Listener);
      return unlisten;
    });
  });

  it("registers listeners before loading the initial snapshot and applies events", async () => {
    render(<Harness />);
    expect(await screen.findByText("water")).toBeInTheDocument();
    const next: AppSnapshot = { ...snapshot, currentReminder: "stand" };
    act(() => callbacks.get("reminder-updated")?.({ payload: next }));
    expect(screen.getByText("stand")).toBeInTheDocument();
    expect(listen).toHaveBeenCalledTimes(4);
    expect(invoke).toHaveBeenCalledWith("get_app_snapshot");
  });

  it("cleans up every event listener", async () => {
    const rendered = render(<Harness />);
    await waitFor(() => expect(listen).toHaveBeenCalledTimes(4));
    rendered.unmount();
    expect(unlisten).toHaveBeenCalledTimes(4);
  });
});
