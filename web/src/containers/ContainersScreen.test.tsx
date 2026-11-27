import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import App from '../App';

const workspaceId = '12345678-1234-4234-8234-123456789abc';
const kegId = '22222222-2222-4222-8222-222222222222';

const container = {
  id: kegId,
  serial_number: 'KEG-0001',
  package_type_id: '33333333-3333-4333-8333-333333333333',
  package_type_name: 'Keg 30 L',
  package_kind: 'keg',
  volume_l: 30,
  status: 'filled',
  version: 1,
  created_at: '2026-11-01T10:00:00Z',
  updated_at: '2026-11-02T10:00:00Z',
};

const list = {
  items: [container],
  next_cursor: null,
  status_counts: [
    { status: 'clean', count: 4 },
    { status: 'filled', count: 1 },
    { status: 'out', count: 0 },
    { status: 'dirty', count: 2 },
    { status: 'maintenance', count: 0 },
    { status: 'retired', count: 0 },
  ],
};

const detail = {
  container,
  events: [
    {
      id: 'e1',
      event_type: 'status_changed',
      old_status: 'clean',
      new_status: 'filled',
      note: 'BBT1',
      occurred_at: '2026-11-02T10:00:00Z',
    },
  ],
};

function json(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  });
}

interface ApiOptions {
  production?: { enabled: boolean; read_only: boolean };
  labels?: () => Response;
}

function stubApi(options: ApiOptions = {}) {
  const production = options.production ?? { enabled: true, read_only: false };
  const fetchMock = vi.fn((url: string, init?: RequestInit) => {
    void init;
    if (url.includes('/v1/me/session')) {
      return Promise.resolve(json({ workspaces: [{ id: workspaceId }] }));
    }
    if (url.includes('/v1/meta')) {
      return Promise.resolve(json({ modules: [{ id: 'production', ...production }] }));
    }
    if (url.includes('/labels')) {
      return Promise.resolve(
        options.labels?.() ??
          new Response('%PDF-1.4', { headers: { 'Content-Type': 'application/pdf' } }),
      );
    }
    if (url.includes('/package-types')) {
      return Promise.resolve(json({ items: [] }));
    }
    if (url.includes(`/containers/${kegId}`)) return Promise.resolve(json(detail));
    if (url.includes('/containers')) return Promise.resolve(json(list));
    return Promise.resolve(json({ code: 'not_found', status: 404, title: 'x', type: 'x' }, 404));
  });
  vi.stubGlobal('fetch', fetchMock);
  return fetchMock;
}

beforeEach(() => {
  localStorage.setItem('demleme.locale', 'en');
  sessionStorage.clear();
  window.location.hash = '#/containers';
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  localStorage.clear();
  sessionStorage.clear();
});

describe('containers screen', () => {
  it('lists kegs with status counts', async () => {
    stubApi();
    render(<App />);
    expect(await screen.findByRole('button', { name: 'KEG-0001' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Dirty\s*2/ })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Clean\s*4/ })).toBeInTheDocument();
    expect(screen.getByRole('heading', { level: 1, name: 'Kegs' })).toBeInTheDocument();
  });

  it('shows the detail with the event log', async () => {
    stubApi();
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'KEG-0001' }));
    const section = await screen.findByRole('region', { name: 'Keg detail' });
    expect(
      await within(section).findByText(/Status changed \(Clean → Filled\)/),
    ).toBeInTheDocument();
    expect(within(section).getByText('BBT1')).toBeInTheDocument();
  });

  it('requests the label PDF for the selected kegs', async () => {
    const fetchMock = stubApi();
    vi.stubGlobal(
      'URL',
      Object.assign(URL, { createObjectURL: () => 'blob:x', revokeObjectURL: () => {} }),
    );
    render(<App />);
    fireEvent.click(await screen.findByRole('checkbox', { name: /KEG-0001/ }));
    fireEvent.change(screen.getByLabelText('Paper'), { target: { value: 'letter' } });
    fireEvent.click(screen.getByRole('button', { name: 'Print labels' }));
    await waitFor(() => {
      const call = fetchMock.mock.calls.find(([url]) => String(url).includes('/labels'));
      expect(call).toBeDefined();
      expect(String(call?.[0])).toContain('paper=letter');
      const init = call?.[1];
      expect(JSON.parse(String(init?.body))).toEqual({
        targets: [{ kind: 'container', id: kegId }],
      });
    });
  });

  it('hides manage actions after the API answers 403 forbidden', async () => {
    stubApi({
      labels: () =>
        json(
          { code: 'forbidden', status: 403, title: 'Forbidden', type: '/errors/forbidden' },
          403,
        ),
    });
    render(<App />);
    fireEvent.click(await screen.findByRole('checkbox', { name: /KEG-0001/ }));
    fireEvent.click(screen.getByRole('button', { name: 'Print labels' }));
    await waitFor(() => expect(screen.queryByRole('button', { name: 'Print labels' })).toBeNull());
    expect(screen.queryByRole('region', { name: 'Add kegs' })).toBeNull();
  });

  it('shows the upgrade message and no navigation entry when Production is off', async () => {
    stubApi({ production: { enabled: false, read_only: false } });
    render(<App />);
    expect(
      await screen.findByRole('heading', { name: 'The Production module is off' }),
    ).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'Kegs' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'KEG-0001' })).toBeNull();
  });

  it('disables write actions with a text reason when Production is read only', async () => {
    stubApi({ production: { enabled: true, read_only: true } });
    render(<App />);
    await screen.findByRole('button', { name: 'KEG-0001' });
    const print = await screen.findByRole('button', { name: 'Print labels' });
    await waitFor(() => expect(print).toHaveAttribute('aria-disabled', 'true'));
    expect(screen.getAllByText(/Read only: the Production module/).length).toBeGreaterThan(0);
  });
});
