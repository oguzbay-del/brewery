import { apiErrorFromResponse, moduleErrorKind, type ModuleErrorKind } from './errors';

type ModuleErrorListener = (kind: ModuleErrorKind) => void;
const moduleErrorListeners = new Set<ModuleErrorListener>();

/** Calls `listener` when a request fails with 403 `module_disabled` or `module_read_only`. */
export function onModuleError(listener: ModuleErrorListener): () => void {
  moduleErrorListeners.add(listener);
  return () => {
    moduleErrorListeners.delete(listener);
  };
}

export interface ApiRequestOptions extends Omit<RequestInit, 'body' | 'headers' | 'method'> {
  method?: string;
  body?: unknown;
  headers?: HeadersInit;
  ifMatch?: string;
  idempotencyKey?: string;
}

const baseUrl = (import.meta.env.VITE_API_BASE_URL ?? '').replace(/\/+$/, '');

function requestUrl(path: string): string {
  if (/^https?:\/\//i.test(path)) return path;
  return `${baseUrl}${path.startsWith('/') ? path : `/${path}`}`;
}

function requestId(): string {
  const cryptoApi = globalThis.crypto;
  if (cryptoApi?.randomUUID) return cryptoApi.randomUUID();
  if (cryptoApi?.getRandomValues) {
    const bytes = cryptoApi.getRandomValues(new Uint8Array(16));
    bytes[6] = (bytes[6]! & 0x0f) | 0x40;
    bytes[8] = (bytes[8]! & 0x3f) | 0x80;
    const hex = Array.from(bytes, (byte) => byte.toString(16).padStart(2, '0')).join('');
    return [
      hex.slice(0, 8),
      hex.slice(8, 12),
      hex.slice(12, 16),
      hex.slice(16, 20),
      hex.slice(20),
    ].join('-');
  }
  throw new Error('Secure UUID generation is unavailable');
}

async function send(path: string, options: ApiRequestOptions, accept: string): Promise<Response> {
  const { method, body, headers: suppliedHeaders, ifMatch, idempotencyKey, ...rest } = options;
  const headers = new Headers(suppliedHeaders);
  headers.set('Accept', accept);
  headers.set('X-Request-ID', requestId());
  if (body !== undefined) headers.set('Content-Type', 'application/json');
  if (ifMatch !== undefined) headers.set('If-Match', ifMatch);
  if (idempotencyKey !== undefined) headers.set('Idempotency-Key', idempotencyKey);

  const response = await fetch(requestUrl(path), {
    ...rest,
    method: method ?? (body === undefined ? 'GET' : 'POST'),
    credentials: 'include',
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (!response.ok) {
    const error = await apiErrorFromResponse(response);
    const kind = moduleErrorKind(error);
    if (kind) moduleErrorListeners.forEach((listener) => listener(kind));
    throw error;
  }
  return response;
}

/** Sends a JSON API request and returns its decoded response. */
export async function apiFetch<T>(path: string, options: ApiRequestOptions = {}): Promise<T> {
  const response = await send(path, options, 'application/json');
  if (response.status === 204) return undefined as T;
  return (await response.json()) as T;
}

/** Sends an API request that answers with a file (for example a PDF) and returns the file. */
export async function apiFetchBlob(
  path: string,
  options: ApiRequestOptions = {},
  accept = 'application/pdf',
): Promise<Blob> {
  return (await send(path, options, accept)).blob();
}
