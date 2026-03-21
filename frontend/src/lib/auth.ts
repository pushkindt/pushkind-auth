import type { FlashAlert } from "../components/AppShell";

export interface HubOption {
  id: number;
  name: string;
}

export interface AuthPageBootstrap {
  shell: {
    alerts: FlashAlert[];
  };
  next: string | null;
  hubs: HubOption[];
}

export function withNext(path: string, next: string | null): string {
  if (!next) {
    return path;
  }

  return `${path}?next=${encodeURIComponent(next)}`;
}
