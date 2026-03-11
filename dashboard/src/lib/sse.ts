export function connectEvents(
  path: string,
  onMessage: (payload: Record<string, unknown>) => void,
  onError?: () => void,
) {
  const source = new EventSource(path);
  source.onmessage = (event) => {
    try {
      onMessage(JSON.parse(event.data));
    } catch {
      return;
    }
  };
  source.onerror = () => {
    onError?.();
  };
  return () => source.close();
}
