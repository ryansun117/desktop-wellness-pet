import { useId, useState, type FormEvent } from "react";
import "./SettingsPanel.css";
import * as commands from "../../services/tauriCommands";
import type { AppSettings, AppSnapshot } from "../../types/appTypes";

interface SettingsPanelProps {
  snapshot: AppSnapshot;
  onSnapshot: (snapshot: AppSnapshot) => void;
  onError: (error: string | null) => void;
  onClose: () => void;
}

export function SettingsPanel({ snapshot, onSnapshot, onError, onClose }: SettingsPanelProps) {
  const [form, setForm] = useState(snapshot.settings);
  const [validation, setValidation] = useState<string[]>([]);
  const [saving, setSaving] = useState(false);

  function patch<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    setForm((current) => ({ ...current, [key]: value }));
  }

  async function save(event: FormEvent) {
    event.preventDefault();
    const issues = validate(form);
    setValidation(issues);
    if (issues.length) return;
    setSaving(true);
    try {
      onSnapshot(await commands.updateSettings(form));
    } catch (reason) {
      onError(commands.describeError(reason));
    } finally {
      setSaving(false);
    }
  }

  async function togglePause() {
    try {
      onSnapshot(snapshot.paused ? await commands.resumeReminders() : await commands.pauseReminders());
    } catch (reason) {
      onError(commands.describeError(reason));
    }
  }

  async function reset() {
    try {
      onSnapshot(await commands.resetSettings());
      setValidation([]);
    } catch (reason) {
      onError(commands.describeError(reason));
    }
  }

  return (
    <section className="settings-panel" aria-labelledby="settings-title">
      <header>
        <div><span className="eyebrow">Wellness Pet</span><h1 id="settings-title">Healthy rhythm</h1></div>
        <button className="icon-button" type="button" aria-label="Close settings" onClick={onClose}>×</button>
      </header>

      <div className="status-strip">
        <Status label="Water" state={snapshot.water.state} />
        <Status label="Stand" state={snapshot.stand.state} />
      </div>

      {validation.length > 0 && <div className="validation" role="alert">{validation.map((issue) => <p key={issue}>{issue}</p>)}</div>}

      <form noValidate onSubmit={(event) => void save(event)}>
        <fieldset>
          <legend>Reminders</legend>
          <Toggle label="Water reminders" checked={form.waterEnabled} onChange={(value) => patch("waterEnabled", value)} />
          <NumberField label="Water interval" value={form.waterIntervalMinutes} suffix="minutes" onChange={(value) => patch("waterIntervalMinutes", value)} />
          <Toggle label="Standing reminders" checked={form.standEnabled} onChange={(value) => patch("standEnabled", value)} />
          <NumberField label="Stand interval" value={form.standIntervalMinutes} suffix="minutes" onChange={(value) => patch("standIntervalMinutes", value)} />
          <NumberField label="Snooze duration" value={form.snoozeDurationMinutes} suffix="minutes" onChange={(value) => patch("snoozeDurationMinutes", value)} />
        </fieldset>

        <fieldset>
          <legend>Quiet hours</legend>
          <Toggle label="Enable quiet hours" checked={form.quietHoursEnabled} onChange={(value) => patch("quietHoursEnabled", value)} />
          <div className="time-row">
            <label>Start<input type="time" value={form.quietHoursStart} onChange={(event) => patch("quietHoursStart", event.target.value)} /></label>
            <label>End<input type="time" value={form.quietHoursEnd} onChange={(event) => patch("quietHoursEnd", event.target.value)} /></label>
          </div>
        </fieldset>

        <fieldset>
          <legend>Desktop</legend>
          <Toggle label="Always on top" checked={form.alwaysOnTop} onChange={(value) => patch("alwaysOnTop", value)} />
          <Toggle label="Launch at login" checked={form.launchAtLogin} onChange={(value) => patch("launchAtLogin", value)} />
        </fieldset>

        <div className="settings-actions">
          <button className="primary" type="submit" disabled={saving}>{saving ? "Saving…" : "Save settings"}</button>
          <button type="button" onClick={() => void togglePause()}>{snapshot.paused ? "Resume reminders" : "Pause reminders"}</button>
          <button className="text-button" type="button" onClick={() => void reset()}>Reset defaults</button>
        </div>
      </form>
    </section>
  );
}

function Toggle({ label, checked, onChange }: { label: string; checked: boolean; onChange: (value: boolean) => void }) {
  const id = useId();
  return <div className="toggle-row"><label htmlFor={id}>{label}</label><input id={id} type="checkbox" checked={checked} onChange={(event) => onChange(event.target.checked)} /></div>;
}

function NumberField({ label, value, suffix, onChange }: { label: string; value: number; suffix: string; onChange: (value: number) => void }) {
  const id = useId();
  return <div className="number-row"><label htmlFor={id}>{label}</label><span className="number-control"><input id={id} type="number" min="1" max="1440" value={value} onChange={(event) => onChange(Number(event.target.value))} /><small>{suffix}</small></span></div>;
}

function Status({ label, state }: { label: string; state: AppSnapshot["water"]["state"] }) {
  let value = "Disabled";
  if (state.status === "due") value = "Due now";
  if (state.status === "snoozed") value = `Snoozed · ${formatTime(state.until)}`;
  if (state.status === "scheduled") value = formatTime(state.dueAt);
  return <div><span>{label}</span><strong>{value}</strong></div>;
}

function formatTime(value: string) {
  return new Intl.DateTimeFormat(undefined, { weekday: "short", hour: "numeric", minute: "2-digit" }).format(new Date(value));
}

function validate(settings: AppSettings) {
  const issues: string[] = [];
  if (settings.waterIntervalMinutes < 1 || settings.waterIntervalMinutes > 1440) issues.push("Water interval must be from 1 to 1440 minutes.");
  if (settings.standIntervalMinutes < 1 || settings.standIntervalMinutes > 1440) issues.push("Stand interval must be from 1 to 1440 minutes.");
  if (settings.snoozeDurationMinutes < 1 || settings.snoozeDurationMinutes > 480) issues.push("Snooze duration must be from 1 to 480 minutes.");
  if (settings.quietHoursStart === settings.quietHoursEnd) issues.push("Quiet-hours start and end must differ.");
  return issues;
}
