import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";
import { Pet } from "./Pet";

it("does not open settings after a mouse drag, but supports keyboard activation", async () => {
  const open = vi.fn();
  render(<Pet mood="idle" onOpenSettings={open} />);
  const pet = screen.getByRole("button");
  fireEvent.click(pet, { detail: 1 });
  expect(open).not.toHaveBeenCalled();
  pet.focus();
  await userEvent.keyboard("{Enter}");
  expect(open).toHaveBeenCalledOnce();
});
