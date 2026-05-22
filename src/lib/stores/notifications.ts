// src/lib/stores/notifications.ts

import { writable } from 'svelte/store';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface Toast {
  id:       string;
  type:     ToastType;
  title:    string;
  message?: string;
  duration: number;
}

function createNotificationStore() {
  const { subscribe, update } = writable<Toast[]>([]);

  function add(toast: Omit<Toast, 'id'>) {
    const id = crypto.randomUUID();
    update(list => [...list, { ...toast, id }]);
    if (toast.duration > 0) {
      setTimeout(() => remove(id), toast.duration);
    }
    return id;
  }

  function remove(id: string) {
    update(list => list.filter(t => t.id !== id));
  }

  return {
    subscribe,
    remove,
    success: (title: string, message?: string) =>
      add({ type: 'success', title, message, duration: 4000 }),
    error: (title: string, message?: string) =>
      add({ type: 'error', title, message, duration: 6000 }),
    warning: (title: string, message?: string) =>
      add({ type: 'warning', title, message, duration: 5000 }),
    info: (title: string, message?: string) =>
      add({ type: 'info', title, message, duration: 4000 }),
  };
}

export const notifications = createNotificationStore();
