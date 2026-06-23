/**
 * useSettingsStore — persistent settings cache backed by Tauri SQLite.
 *
 * Known keys:
 *   "default_view"      → "grid" | "list"
 *   "launch_count"      → number (informational)
 *
 * Usage:
 *   const { settings, setSetting, loadSettings } = useSettingsStore();
 */

import { create } from "zustand";
import { getAllSettings, setSetting as apiSetSetting } from "@/lib/api";

interface SettingsStore {
  /** Full key-value map, loaded from the DB */
  settings: Record<string, string>;
  loading:  boolean;
  /** Load all settings from the DB into the cache */
  loadSettings: () => Promise<void>;
  /** Update a single setting — persists to DB and updates cache */
  setSetting: (key: string, value: string) => Promise<void>;
  /** Read a single cached value with an optional default */
  getSetting: (key: string, defaultValue?: string) => string | undefined;
}

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  settings: {},
  loading:  false,

  loadSettings: async () => {
    set({ loading: true });
    try {
      const data = await getAllSettings();
      set({ settings: data });
    } finally {
      set({ loading: false });
    }
  },

  setSetting: async (key, value) => {
    await apiSetSetting(key, value);
    set((state) => ({
      settings: { ...state.settings, [key]: value },
    }));
  },

  getSetting: (key, defaultValue) => {
    return get().settings[key] ?? defaultValue;
  },
}));
