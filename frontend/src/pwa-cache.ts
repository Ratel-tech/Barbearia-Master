export function isCacheableRequest(request: Request, origin: string) {
  if (request.method !== "GET") return false;
  const url = new URL(request.url);
  return url.origin === origin && !url.pathname.startsWith("/api/");
}

export function shouldCacheResponse(response: Response) {
  return response.ok;
}
