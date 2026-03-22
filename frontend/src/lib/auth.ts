export interface HubOption {
  id: number;
  name: string;
}

export function getNextFromLocation(): string | null {
  const next = new URLSearchParams(window.location.search).get("next");

  return next && next.length > 0 ? next : null;
}

export function withNext(path: string, next: string | null): string {
  if (!next) {
    return path;
  }

  return `${path}?next=${encodeURIComponent(next)}`;
}
