import { afterEach, describe, expect, it, vi } from 'vitest';
import { apiFetch } from './client';
import { ApiError, StaleVersionError } from './errors';

afterEach(() => {
  vi.unstubAllGlobals();
});

function response(status: number, body: unknown, headers: Record<string, string> = {}): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/problem+json', ...headers },
  });
}

describe('apiFetch', () => {
  it('returns decoded JSON on success and includes credentials and a request id', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ id: 'recipe-1' }), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(apiFetch<{ id: string }>('/v1/recipes')).resolves.toEqual({ id: 'recipe-1' });
    const [, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    const headers = new Headers(init.headers);
    expect(init.credentials).toBe('include');
    expect(headers.get('Accept')).toBe('application/json');
    expect(headers.get('X-Request-ID')).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);
  });

  it('parses a 401 problem into ApiError', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(response(401, {
      type: '/errors/unauthenticated',
      title: 'Unauthenticated',
      status: 401,
      code: 'unauthenticated',
      request_id: 'request-401',
    })));

    await expect(apiFetch('/v1/me')).rejects.toMatchObject({
      name: 'ApiError',
      status: 401,
      code: 'unauthenticated',
      requestId: 'request-401',
    });
  });

  it('maps 412 to StaleVersionError and preserves conflicting fields', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(response(412, {
      type: '/errors/version_mismatch',
      title: 'Version mismatch',
      status: 412,
      code: 'version_mismatch',
      request_id: 'request-412',
      current: { version: 4, conflicting_fields: ['efficiency_pct', 'batch_volume_l'] },
    })));

    await expect(apiFetch('/v1/recipes/1', { method: 'PATCH', body: {} })).rejects.toMatchObject({
      name: 'StaleVersionError',
      status: 412,
      current: { version: 4, conflicting_fields: ['efficiency_pct', 'batch_volume_l'] },
    });
  });

  it('parses a 409 conflict', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(response(409, {
      type: '/errors/conflict',
      title: 'Conflict',
      status: 409,
      code: 'conflict',
      request_id: 'request-409',
    })));

    await expect(apiFetch('/v1/recipes')).rejects.toBeInstanceOf(ApiError);
    await expect(apiFetch('/v1/recipes')).rejects.toMatchObject({ status: 409, code: 'conflict' });
  });

  it('lets network errors propagate without disguising them as HTTP errors', async () => {
    const failure = new TypeError('network unavailable');
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(failure));
    await expect(apiFetch('/v1/me')).rejects.toBe(failure);
  });

  it('sets JSON body, If-Match and Idempotency-Key headers', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ ok: true }), { status: 200 }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await apiFetch('/v1/recipes/1', {
      method: 'PATCH',
      body: { name: 'Pale ale' },
      ifMatch: '"v3"',
      idempotencyKey: 'idem-123',
    });

    const [, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    const headers = new Headers(init.headers);
    expect(headers.get('Content-Type')).toBe('application/json');
    expect(headers.get('If-Match')).toBe('"v3"');
    expect(headers.get('Idempotency-Key')).toBe('idem-123');
    expect(init.body).toBe(JSON.stringify({ name: 'Pale ale' }));
  });
});
