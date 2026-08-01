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
    <button className={`pet pet-${mood}`} type="button" aria-label={`${moodLabels[mood]}. Open settings.`} onClick={onOpenSettings}>
      <svg viewBox="0 0 180 170" role="img" aria-hidden="true">
        <defs>
          <linearGradient id="pet-body" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0" stopColor="#8ce0d5" />
            <stop offset="1" stopColor="#45aab3" />
          </linearGradient>
          <filter id="pet-shadow" x="-30%" y="-30%" width="160%" height="180%">
            <feDropShadow dx="0" dy="7" stdDeviation="7" floodColor="#153945" floodOpacity=".24" />
          </filter>
        </defs>
        <ellipse className="pet-ground" cx="90" cy="150" rx="56" ry="10" />
        <g className="pet-creature" filter="url(#pet-shadow)">
          <path className="pet-ear pet-ear-left" d="M45 56 36 22 70 43Z" />
          <path className="pet-ear pet-ear-right" d="m110 43 34-21-9 35Z" />
          <path className="pet-body" d="M42 79c0-31 20-48 48-48s49 17 49 48v35c0 27-21 43-49 43s-48-16-48-43Z" />
          <path className="pet-belly" d="M62 111c0-18 12-29 28-29s29 11 29 29v8c0 17-12 27-29 27s-28-10-28-27Z" />
          <g className="pet-face">
            <ellipse className="pet-eye pet-eye-left" cx="71" cy="76" rx="6" ry="8" />
            <ellipse className="pet-eye pet-eye-right" cx="109" cy="76" rx="6" ry="8" />
            <path className="pet-mouth" d="M82 91q8 8 16 0" />
            <path className="pet-cheek pet-cheek-left" d="M57 92h10" />
            <path className="pet-cheek pet-cheek-right" d="M113 92h10" />
          </g>
          <path className="pet-arm pet-arm-left" d="M52 105q-19 5-24 21" />
          <path className="pet-arm pet-arm-right" d="M128 105q19 5 24 21" />
          <g className="water-drop"><path d="M148 50c0 9-12 9-12 0 0-4 6-12 6-12s6 8 6 12Z" /></g>
          <g className="sleep-mark"><path d="m138 45 14-1-13 15 15-1" /></g>
          <g className="sparkles"><path d="M34 55v-14m-7 7h14M150 78V62m-8 8h16" /></g>
        </g>
      </svg>
    </button>
  );
}

