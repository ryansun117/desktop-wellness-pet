import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ReminderBubble } from "./ReminderBubble";

describe("ReminderBubble", () => {
  it("renders the current reminder and routes all actions", () => {
    const done = vi.fn();
    const snooze = vi.fn();
    const skip = vi.fn();
    render(<ReminderBubble kind="water" onDone={done} onSnooze={snooze} onSkip={skip} />);

    expect(screen.getByText("Time for some water")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Done" }));
    fireEvent.click(screen.getByRole("button", { name: "Snooze" }));
    fireEvent.click(screen.getByRole("button", { name: "Skip" }));
    expect(done).toHaveBeenCalledWith("water");
    expect(snooze).toHaveBeenCalledWith("water");
    expect(skip).toHaveBeenCalledWith("water");
  });
});

