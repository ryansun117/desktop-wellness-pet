import "./ReminderBubble.css";
import type { ReminderKind } from "../../types/appTypes";

interface ReminderBubbleProps {
  kind: ReminderKind | null;
  onDone: (kind: ReminderKind) => void;
  onSnooze: (kind: ReminderKind) => void;
  onSkip: (kind: ReminderKind) => void;
}

export function ReminderBubble({ kind, onDone, onSnooze, onSkip }: ReminderBubbleProps) {
  if (!kind) return null;
  const message = kind === "water" ? "Time for some water" : "Time to stand and stretch";
  return (
    <section className="reminder-bubble" role="alert" aria-live="assertive">
      <p>{message}</p>
      <div className="reminder-actions">
        <button className="primary" type="button" onClick={() => onDone(kind)}>Done</button>
        <button type="button" onClick={() => onSnooze(kind)}>Snooze</button>
        <button type="button" onClick={() => onSkip(kind)}>Skip</button>
      </div>
    </section>
  );
}

