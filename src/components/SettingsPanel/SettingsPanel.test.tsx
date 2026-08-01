import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { snapshot } from "../../test/fixtures";
import { SettingsPanel } from "./SettingsPanel";

vi.mock("../../services/tauriCommands", async (importOriginal) => {
  const original = await importOriginal<typeof import("../../services/tauriCommands")>();
  return { ...original, updateSettings: vi.fn() };
});

describe("SettingsPanel", () => {
  it("shows useful validation and does not submit invalid settings", () => {
    render(<SettingsPanel snapshot={snapshot} onSnapshot={vi.fn()} onError={vi.fn()} onClose={vi.fn()} />);
    const water = screen.getByLabelText("Water interval");
    fireEvent.change(water, { target: { value: "0" } });
    fireEvent.click(screen.getByRole("button", { name: "Save settings" }));
    expect(screen.getByRole("alert")).toHaveTextContent("Water interval must be from 1 to 1440 minutes.");
  });
});

