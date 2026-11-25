import { afterEach, describe, expect, it, vi } from 'vitest';
import { apiFetch } from './client';
import { ApiError, StaleVersionError } from './errors';

afterEach(() => {
  vi.unstubAllGlobals();
});

function problem(status: number, code: string, extra: Record<string, unknown> = {}): Response {
  return new Response(
    JSON.stringify({
      type: `/errors/${code}`,
      title: code,
      status,
      code,
      request_id: `request-${status}`,
      ...extra,
    }),
    { status, headers: { 'content-type': 'application/problem+json' } },
  );
}

describe('apiFetch', () => {
  it('returns JSON and sets credentials and request ID', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(new Response(JSON.stringify({ id: 'recipe-1' }), { status: 200 }));
    vi.stubGlobal('fetch', fetchMock);

    await expect(apiFetch<{ id: string }>('/v1/recipes')).resolves.toEqual({
      id: 'recipe-1',
    });
    const [, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    const headers = new Headers(init.headers);
    expect(init.credentials).toBe('include');
    expect(headers.get('Accept')).toBe('application/json');
    expect(headers.get('X-Request-ID')).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i,
    );
  });

  it('parses 401 errors', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(problem(401, 'unauthenticated')));
    await expect(apiFetch('/v1/me')).rejects.toMatchObject({
      name: 'ApiError',
      status: 401,
      code: 'unauthenticated',
      requestId: 'request-401',
    });
  });

  it('maps 412 to StaleVersionError and retains conflicting fields', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        problem(412, 'version_mismatch', {
          current: {
            version: 4,
            conflicting_fields: ['efficiency_pct', 'batch_volume_l'],
          },
        }),
      ),
    );
    const request = apiFetch('/v1/recipes/1', { method: 'PATCH', body: {} });
    await expect(request).rejects.toBeInstanceOf(StaleVersionError);
    await expect(request).rejects.toMatchObject({
      status: 412,
      current: {
        version: 4,
        conflicting_fields: ['efficiency_pct', 'batch_volume_l'],
      },
    });
  });

  it('parses 409 conflicts', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(problem(409, 'conflict')));
    const request = apiFetch('/v1/recipes');
    await expect(request).rejects.toBeInstanceOf(ApiError);
    await expect(request).rejects.toMatchObject({ status: 409, code: 'conflict' });
  });

  it('preserves network errors', async () => {
    const failure = new TypeError('network unavailable');
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(failure));
    await expect(apiFetch('/v1/me')).rejects.toBe(failure);
  });

  it('sets JSON, If-Match and Idempotency-Key headers', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(new Response(JSON.stringify({ ok: true }), { status: 200 }));
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
