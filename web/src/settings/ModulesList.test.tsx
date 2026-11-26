import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { translate } from '../i18n/catalog';
import { WorkspaceProvider } from '../workspace/WorkspaceProvider';
import { ModulesList } from './ModulesList';

const workspaceId = '12345678-1234-4234-8234-123456789abc';

function json(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  });
}

function show() {
  render(
    <WorkspaceProvider>
      <ModulesList />
    </WorkspaceProvider>,
  );
}

function text(key: string) {
  return translate(key, 'en');
}

beforeEach(() => {
  localStorage.setItem('demleme.locale', 'en');
  sessionStorage.clear();
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  localStorage.clear();
  sessionStorage.clear();
});

describe('module settings', () => {
  it('shows loading while the session is pending', () => {
    vi.stubGlobal('fetch', vi.fn(() => new Promise<Response>(() => {})));
    show();
    expect(screen.getByRole('status').textContent).toBe(text('ui.modules.loading'));
  });

  it('shows all three module states', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(json({ workspaces: [{ id: workspaceId }] }))
      .mockResolvedValueOnce(
        json({
          modules: [
            { id: 'core', enabled: true, read_only: false },
            { id: 'devices', enabled: false, read_only: false },
            { id: 'stock', enabled: true, read_only: true },
          ],
        }),
      );
    vi.stubGlobal('fetch', fetchMock);
    show();
    const rows = await screen.findAllByRole('listitem');
    expect(rows).toHaveLength(3);
    expect(rows[0]?.textContent).toContain(text('ui.modules.on'));
    expect(rows[1]?.textContent).toContain(text('ui.modules.off'));
    expect(rows[2]?.textContent).toContain(text('ui.modules.read_only'));
    expect(fetchMock.mock.calls[1]?.[0]).toContain(workspaceId);
  });

  it('shows an empty workspace state', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(json({ workspaces: [] })));
    show();
    expect(await screen.findByText(text('ui.modules.empty'))).toBeTruthy();
  });

  it('shows the sign-in error for a 401 session response', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(json({}, 401)));
    show();
    const alert = await screen.findByRole('alert');
    expect(alert.textContent).toContain(text('ui.modules.sign_in_required'));
  });

  it('shows the membership error for a 404 module response', async () => {
    const fetchMock = vi.fn();
    fetchMock.mockResolvedValueOnce(json({ workspaces: [{ id: workspaceId }] }));
    fetchMock.mockResolvedValueOnce(json({}, 404));
    vi.stubGlobal('fetch', fetchMock);
    show();
    const alert = await screen.findByRole('alert');
    expect(alert.textContent).toContain(text('ui.modules.not_member'));
  });
});
