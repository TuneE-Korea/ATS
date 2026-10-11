import { TARGET_LANGUAGES } from "../lib/languages";
import { inputClass } from "./Section";

export function LanguageSelect({ value, onChange }: { value: string; onChange: (code: string) => void }) {
  return (
    <select className={inputClass} value={value} onChange={(e) => onChange(e.target.value)}>
      {TARGET_LANGUAGES.map((l) => (
        <option key={l.code} value={l.code}>
          {l.label}
        </option>
      ))}
    </select>
  );
}
