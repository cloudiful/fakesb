export function normalizeApiBase(value?: string) {
  return String(value || '/').replace(/\/$/, '')
}
