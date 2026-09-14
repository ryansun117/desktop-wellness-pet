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
      <p>
        {kind === "water" && (
          <svg className="reminder-water-icon" viewBox="0 0 16 20" aria-hidden="true">
            <path d="M8 1C6 5 1 10 1 13a7 7 0 0 0 14 0C15 10 10 5 8 1Z" fill="#429fcb" />
            <path d="M4 12q-1 4 3 5" fill="none" stroke="#d9f5ff" strokeWidth="1.5" strokeLinecap="round" />
          </svg>
        )}
        {message}
      </p>
      <div className="reminder-actions">
        <button className="primary" type="button" onClick={() => onDone(kind)}>Done</button>
        <button type="button" onClick={() => onSnooze(kind)}>Snooze</button>
        <button type="button" onClick={() => onSkip(kind)}>Skip</button>
      </div>
    </section>
  );
}
