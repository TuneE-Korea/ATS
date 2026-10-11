import { create } from "zustand";
import { errorMessage, ipc } from "../lib/ipc";
import type { Settings } from "../lib/types";

// saved: Rust에 저장된 값, draft: 폼에서 편집 중인 값.
// 둘을 나눠 두면 "저장 안 된 변경 있음" 표시와 되돌리기가 간단해진다.
interface SettingsState {
  saved: Settings | null;
  draft: Settings | null;
  saving: boolean;
  error: string | null;
  load: () => Promise<void>;
  edit: (patch: Partial<Settings>) => void;
  reset: () => void;
  save: () => Promise<boolean>;
}

export const useSettings = create<SettingsState>((set, get) => ({
  saved: null,
  draft: null,
  saving: false,
  error: null,

  load: async () => {
    try {
      const s = await ipc.getSettings();
      set({ saved: s, draft: s, error: null });
    } catch (e) {
      set({ error: errorMessage(e) });
    }
  },

  edit: (patch) => {
    const { draft } = get();
    if (draft) set({ draft: { ...draft, ...patch } });
  },

  reset: () => set((s) => ({ draft: s.saved, error: null })),

  save: async () => {
    const { draft } = get();
    if (!draft) return false;
    set({ saving: true, error: null });
    try {
      await ipc.saveSettings(draft);
      set({ saved: draft, saving: false });
      return true;
    } catch (e) {
      set({ saving: false, error: errorMessage(e) });
      return false;
    }
  },
}));

export function isDirty(saved: Settings | null, draft: Settings | null): boolean {
  if (!saved || !draft) return false;
  return (Object.keys(saved) as (keyof Settings)[]).some((k) => saved[k] !== draft[k]);
}
