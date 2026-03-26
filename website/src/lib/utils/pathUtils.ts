/**
 * Prepends the Astro base URL to internal paths.
 * External URLs (http/https) and anchors (#) are returned unchanged.
 *
 * This allows config files to use clean root-relative paths like `/getting-started/`
 * while the actual base path (e.g. `/spring-batch-rs`) is managed solely in astro.config.mjs.
 */
export function withBase(path: string): string {
  if (!path || path.startsWith("http") || path.startsWith("//") || path.startsWith("#")) {
    return path;
  }
  const base = import.meta.env.BASE_URL.replace(/\/$/, "");
  return base + (path.startsWith("/") ? path : "/" + path);
}
