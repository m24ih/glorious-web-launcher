const rawBase = import.meta.env.VITE_GLORIOUS_API ?? 'http://127.0.0.1:25820';
const rawWs = import.meta.env.VITE_GLORIOUS_WS;

export const API_BASE = rawBase.replace(/\/+$/, '');
export const WS_URL = rawWs ?? `${API_BASE.replace(/^http/, 'ws')}/ws`;

const JSON_HEADERS = { 'content-type': 'application/json' };

export class ApiError extends Error {
  status: number;

  constructor(message: string, status = 0) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
  }
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  let response: Response;
  try {
    response = await fetch(`${API_BASE}${path}`, {
      ...init,
      signal: AbortSignal.timeout(6000),
    });
  } catch (error) {
    const message =
      error instanceof DOMException && error.name === 'TimeoutError'
        ? 'The local service did not respond in time.'
        : 'The local Glorious service is not reachable.';
    throw new ApiError(message);
  }

  if (!response.ok) {
    let message = `${response.status} ${response.statusText}`;
    try {
      const body = (await response.json()) as { error?: string };
      if (body.error) message = body.error;
    } catch {
      // Keep the status line when the body is not JSON.
    }
    throw new ApiError(message, response.status);
  }

  return (await response.json()) as T;
}

export function fetchDevices() {
  return request<import('./types').DeviceList>('/api/devices');
}

export function fetchDevice(slug: string) {
  return request<import('./types').DeviceView>(
    `/api/device/${encodeURIComponent(slug)}`,
  );
}

export function applySettings(
  slug: string,
  patch: import('./types').SettingsPatch,
) {
  return request<import('./types').ApplyResult>(
    `/api/device/${encodeURIComponent(slug)}/apply`,
    {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify(patch),
    },
  );
}

export function fetchHealth() {
  return request<import('./types').HealthInfo>('/api/health');
}

export function connectWs(
  onMessage: (event: import('./types').WsEvent) => void,
  onStatus?: (status: 'live' | 'offline') => void,
): () => void {
  const socket = new WebSocket(WS_URL);
  socket.addEventListener('open', () => onStatus?.('live'));
  socket.addEventListener('close', () => onStatus?.('offline'));
  socket.addEventListener('error', () => onStatus?.('offline'));
  socket.addEventListener('message', (message) => {
    try {
      onMessage(JSON.parse(message.data) as import('./types').WsEvent);
    } catch {
      // Ignore malformed service messages.
    }
  });
  return () => socket.close();
}
