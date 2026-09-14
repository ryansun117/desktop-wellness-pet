import "./Pet.css";

export type PetMood = "idle" | "waterDue" | "standDue" | "snoozed" | "celebrating" | "paused";

interface PetProps {
  mood: PetMood;
  onOpenSettings: () => void;
}

const moodLabels: Record<PetMood, string> = {
  idle: "Wellness Pet is content",
  waterDue: "Wellness Pet wants you to drink water",
  standDue: "Wellness Pet wants you to stand and stretch",
  snoozed: "Wellness Pet is snoozing",
  celebrating: "Wellness Pet is celebrating",
  paused: "Wellness Pet is resting while reminders are paused",
};

export function Pet({ mood, onOpenSettings }: PetProps) {
  return (
    <button
      className={`pet pet-${mood}`}
      type="button"
      data-tauri-drag-region
      aria-label={`${moodLabels[mood]}. Drag to move; press Enter for settings.`}
      title="Drag to move · Settings below"
      onClick={(event) => {
        // Native dragging can end with a mouse click; only keyboard/assistive
        // activation opens settings here. The Settings button handles clicks.
        if (event.detail === 0) onOpenSettings();
      }}
    >
      <svg viewBox="0 0 180 170" role="img" aria-hidden="true">
        <defs>
          <radialGradient id="pet-fur" cx="38%" cy="25%" r="80%">
            <stop offset="0" stopColor="#f4ce8a" />
            <stop offset=".6" stopColor="#dba25c" />
            <stop offset="1" stopColor="#b5773e" />
          </radialGradient>
          <radialGradient id="pet-ruff" cx="50%" cy="30%" r="75%">
            <stop offset="0" stopColor="#fff3d9" />
            <stop offset="1" stopColor="#edc38a" />
          </radialGradient>
          <linearGradient id="pet-iris" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0" stopColor="#8c4d23" />
            <stop offset="1" stopColor="#edb750" />
          </linearGradient>
          <filter id="pet-shadow" x="-25%" y="-25%" width="150%" height="160%">
            <feDropShadow dx="0" dy="4" stdDeviation="3" floodColor="#604322" floodOpacity=".18" />
          </filter>
        </defs>
        <ellipse className="pet-ground" cx="90" cy="156" rx="55" ry="6" />
        <g className="pet-creature" filter="url(#pet-shadow)">
          <path className="pet-tail" d="M120 143c31 7 40-7 34-22-4-10-14-6-12 2 3 9-10 11-18 6" />
          <path className="pet-body" d="M55 108q-10 14-9 28l-4 4 7 2-3 6q18 10 44 7 27 3 43-8l-5-5 6-3-5-6q1-14-10-25Z" />
          <path className="pet-belly" d="M62 108q28-16 56 0l-1 20-7-3-2 12-9-5-9 12-9-12-9 5-2-12-7 3Z" />
          <g className="pet-arm pet-arm-left">
            <path d="M58 128q-6 9-6 19c0 11 27 11 27 0l-1-18" />
            <path className="pet-toes" d="m61 149 0 5m8-5 0 5" />
          </g>
          <g className="pet-arm pet-arm-right">
            <path d="m103 129-1 18c0 11 27 11 27 0 0-10-6-19-6-19" />
            <path className="pet-toes" d="m111 149 0 5m8-5 0 5" />
          </g>
          <path className="pet-ear" d="M42 57Q31 15 46 24l27 19m35 0 27-19q15-9 4 33" />
          <path className="pet-inner-ear" d="m44 34 4 22 14-11Zm92 0-18 11 14 11Z" />
          <path className="pet-head" d="M48 48q12-13 29-12l8-6 4 5 9-6 1 7q22-1 34 13l7-2-1 10q10 8 9 20l7 7-7 2 7 10-10 1 3 9-11-1q-11 19-47 20-34-1-48-20l-10 1 3-9-10-1 7-10-7-2 7-7q-1-12 9-20l-1-10Z" />
          <path className="pet-face-ruff" d="M38 81q10-9 20 0 13 9 32 8 19 1 32-8 10-9 20 0l-6 9 7 2-9 6 3 4-12 1q-11 15-35 15t-35-15l-12-1 3-4-9-6 7-2Z" />
          <path className="pet-fur-detail" d="m76 45 5 10m9-13v11m14-8-5 10M43 69l8 2m78 0 8-2" />
          <g className="pet-face">
            <g className="pet-eye pet-eye-left">
              <ellipse className="pet-eye-rim" cx="66" cy="76" rx="14" ry="16" />
              <ellipse className="pet-iris" cx="66" cy="77" rx="11.5" ry="13.5" />
              <ellipse className="pet-pupil" cx="67" cy="76" rx="6.5" ry="10" />
              <circle className="pet-eye-shine" cx="62" cy="70" r="4" />
              <circle className="pet-eye-shine" cx="71" cy="82" r="1.8" />
            </g>
            <g className="pet-eye pet-eye-right">
              <ellipse className="pet-eye-rim" cx="114" cy="76" rx="14" ry="16" />
              <ellipse className="pet-iris" cx="114" cy="77" rx="11.5" ry="13.5" />
              <ellipse className="pet-pupil" cx="113" cy="76" rx="6.5" ry="10" />
              <circle className="pet-eye-shine" cx="109" cy="70" r="4" />
              <circle className="pet-eye-shine" cx="118" cy="82" r="1.8" />
            </g>
            <ellipse className="pet-cheek" cx="52" cy="93" rx="8" ry="4" />
            <ellipse className="pet-cheek" cx="128" cy="93" rx="8" ry="4" />
            <ellipse className="pet-muzzle" cx="82" cy="96" rx="12" ry="9" />
            <ellipse className="pet-muzzle" cx="98" cy="96" rx="12" ry="9" />
            <path className="pet-nose" d="M84 87q6-3 12 0 1 3-6 7-7-4-6-7Z" />
            <path className="pet-mouth" d="M90 94v4m-7 1q4 4 7-1 3 5 7 1" />
            <path className="pet-whiskers" d="m68 94-19-3m19 8-20 3m64-8 19-3m-19 8 20 3" />
          </g>
          <g className="sleep-mark"><path d="m148 46 10-1-9 11 11-1" /></g>
          <g className="sparkles"><path d="M23 61V49m-6 6h12m126 48V91m-6 6h12" /></g>
        </g>
      </svg>
    </button>
  );
}
