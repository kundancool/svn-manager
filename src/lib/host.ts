/** Host of an svn URL for keychain lookups; "" for file:// and local repos. */
export function host_of(url: string): string {
  if (!url || url.startsWith("file://")) return "";
  const rest = url.split("://")[1];
  if (!rest) return "";
  const authority = rest.split("/")[0] ?? "";
  const host = authority.split("@").pop()?.split(":")[0] ?? "";
  return host;
}
