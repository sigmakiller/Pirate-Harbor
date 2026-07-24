/**
 * useToastStore — Zustand queue for toast notifications.
 *
 * Usage:
 *   const { addToast } = useToastStore();
 *   addToast({ message: "Game deleted", type: "success" });
 *   addToast({ message: "Failed to save", type: "error" });
 */

import { create } from "zustand";

export type ToastType = "success" | "error" | "info" | "achievement";


export interface ToastAction {
  label:   string;
  onClick: () => void;
}

export interface Toast {
  id:      string;
  message: string;
  type:    ToastType;
  action?: ToastAction;  // T57: optional CTA button
  /** Override auto-dismiss duration in ms (default 4000) */
  duration?: number;
}

interface ToastState {
  toasts:     Toast[];
  addToast:   (toast: Omit<Toast, "id">) => void;
  removeToast: (id: string) => void;
}

let _counter = 0;

export const useToastStore = create<ToastState>((set) => ({
  toasts: [],

  addToast: (toast) => {
    const id = `toast-${++_counter}`;
    set(state => ({ toasts: [...state.toasts, { ...toast, id }] }));
    // Auto-dismiss (respect per-toast duration, default 4 s)
    setTimeout(() => {
      set(state => ({ toasts: state.toasts.filter(t => t.id !== id) }));
    }, toast.duration ?? 4000);
  },

  removeToast: (id) => {
    set(state => ({ toasts: state.toasts.filter(t => t.id !== id) }));
  },
}));
