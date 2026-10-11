import { withApp } from "../lib/apps";
import { create } from "zustand";
import { errorMessage, ipc } from "../lib/ipc";
import { move } from "../lib/profiles";
import type { Config, Profile, Secrets, Settings } from "../lib/types";

// saved: Rust에 저장된 값, draft: 폼에서 편집 중인 값.
// 둘을 나눠 두면 "저장 안 된 변경 있음" 표시와 되돌리기가 간단해진다.
interface ConfigState {
  saved: Config | null;
  draft: Config | null;
  saving: boolean;
  error: string | null;
  load: () => Promise<void>;
  editSettings: (patch: Partial<Settings>) => void;
  editSecrets: (patch: Partial<Secrets>) => void;
  addProfile: (profile: Profile) => void;
  /** 이 앱에서 데스크톱 미리보기를 쓰도록 목록에 넣고, 꺼져 있던 실시간 미리보기도 켠다. */
  enableDesktopPreview: (process: string) => void;
  updateProfile: (index: number, patch: Partial<Profile>) => void;
  removeProfile: (index: number) => void;
  moveProfile: (index: number, dir: -1 | 1) => void;
  reset: () => void;
  save: () => Promise<boolean>;
  regeneratePairingToken: () => Promise<void>;
}

export const useConfig = create<ConfigState>((set, get) => {
  /** draft.settings.profiles만 바꾸는 공통 헬퍼. */
  const setProfiles = (fn: (profiles: Profile[]) => Profile[]) => {
    const { draft } = get();
    if (draft) set({ draft: { ...draft, settings: { ...draft.settings, profiles: fn(draft.settings.profiles) } } });
  };

  return {
    saved: null,
    draft: null,
    saving: false,
    error: null,

    load: async () => {
      try {
        const c = await ipc.getConfig();
        set({ saved: c, draft: c, error: null });
      } catch (e) {
        set({ error: errorMessage(e) });
      }
    },

    editSettings: (patch) => {
      const { draft } = get();
      if (draft) set({ draft: { ...draft, settings: { ...draft.settings, ...patch } } });
    },

    editSecrets: (patch) => {
      const { draft } = get();
      if (draft) set({ draft: { ...draft, secrets: { ...draft.secrets, ...patch } } });
    },

    addProfile: (profile) => setProfiles((ps) => [...ps, profile]),
    enableDesktopPreview: (process) => {
      const settings = get().draft?.settings;
      if (!settings) return;
      get().editSettings({
        mode: "realtime",
        desktopRealtime: true,
        desktopRealtimeApps: withApp(settings.desktopRealtimeApps, process),
      });
    },
    updateProfile: (index, patch) => setProfiles((ps) => ps.map((p, i) => (i === index ? { ...p, ...patch } : p))),
    removeProfile: (index) => setProfiles((ps) => ps.filter((_, i) => i !== index)),
    moveProfile: (index, dir) => setProfiles((ps) => move(ps, index, dir)),

    reset: () => set((s) => ({ draft: s.saved, error: null })),

    // 토큰은 즉시 저장된다. 편집 중인 다른 값은 건드리지 않고 토큰만 바꿔 끼운다.
    regeneratePairingToken: async () => {
      try {
        const next = await ipc.regeneratePairingToken();
        const token = next.secrets.pairingToken;
        set((s) => ({
          saved: s.saved && { ...s.saved, secrets: { ...s.saved.secrets, pairingToken: token } },
          draft: s.draft && { ...s.draft, secrets: { ...s.draft.secrets, pairingToken: token } },
        }));
      } catch (e) {
        set({ error: errorMessage(e) });
      }
    },

    save: async () => {
      const { draft } = get();
      if (!draft) return false;
      set({ saving: true, error: null });
      try {
        await ipc.saveConfig(draft);
        set({ saved: draft, saving: false });
        return true;
      } catch (e) {
        set({ saving: false, error: errorMessage(e) });
        return false;
      }
    },
  };
});

/** 설정 객체는 작고 순서가 고정이라 JSON 비교로 충분하다. */
export function isDirty(saved: Config | null, draft: Config | null): boolean {
  if (!saved || !draft) return false;
  return JSON.stringify(saved) !== JSON.stringify(draft);
}
