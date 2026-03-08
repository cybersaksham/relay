export function subscribeToSse(
  url: string,
  onMessage: (event: MessageEvent<string>) => void,
  onError?: (event: Event) => void,
) {
  const source = new EventSource(url);
  source.onmessage = onMessage;
  source.addEventListener("terminal", onMessage as EventListener);
  source.addEventListener("status", onMessage as EventListener);
  if (onError) {
    source.onerror = onError;
  }
  return () => source.close();
}
