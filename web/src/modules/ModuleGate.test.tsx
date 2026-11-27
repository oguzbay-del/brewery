import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import App from '../App';
import { apiFetch } from '../api/client';
import { translate } from '../i18n/catalog';
import { ModuleWriteButton } from './ModuleGate';

const workspaceId = '12345678-1234-4234-8234-123456789abc';

function json(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  });
}

function stubApi(core: { enabled: boolean; read_only: boolean }) {
  vi.stubGlobal(
    'fetch',
    vi.fn((url: string) => {
      if (url.includes('/v1/me/session'))
        return Promise.resolve(json({ workspaces: [{ id: workspaceId }] }));
      if (url.includes('/v1/meta')) {
        return Promise.resolve(json({ modules: [{ id: 'core', ...core }] }));
      }
      return Promise.resolve(json({ code: 'not_found', status: 404, title: 'x', type: 'x' }, 404));
    }),
  );
}

const text = (key: string, params = {}) => translate(key, 'en', params);

beforeEach(() => {
  localStorage.setItem('demleme.locale', 'en');
  sessionStorage.clear();
  window.location.hash = '#/';
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  localStorage.clear();
  sessionStorage.clear();
});

describe('module gating', () => {
  it('hides the entries of a module that is off', async () => {
    stubApi({ enabled: false, read_only: false });
    render(<App />);
    expect(await screen.findByRole('link', { name: 'Settings' })).toBeInTheDocument();
    await vi.waitFor(() => expect(screen.queryByRole('link', { name: /Recipes/ })).toBeNull());
    expect(screen.queryByRole('link', { name: /Batches/ })).toBeNull();
    expect(screen.getByRole('link', { name: 'Home' })).toBeInTheDocument();
  });

  it('keeps a read-only entry and marks it with text', async () => {
    stubApi({ enabled: true, read_only: true });
    render(<App />);
    const link = await screen.findByRole('link', { name: /Recipes\s*\(Read only\)/ });
    expect(link).toHaveTextContent('(Read only)');
  });

  it('shows the upgrade message for the direct URL of an off module', async () => {
    window.location.hash = '#/recipes';
    stubApi({ enabled: false, read_only: false });
    render(<App />);
    expect(
      await screen.findByRole('heading', {
        name: text('ui.modules.upgrade_title', { module: 'Core' }),
      }),
    ).toBeInTheDocument();
    expect(screen.getByRole('link', { name: text('ui.modules.upgrade_link') })).toHaveAttribute(
      'href',
      '#/settings',
    );
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('shows a banner on a read-only module screen', async () => {
    window.location.hash = '#/batches';
    stubApi({ enabled: true, read_only: true });
    render(<App />);
    expect(
      await screen.findByText(text('ui.modules.read_only_banner', { module: 'Core' })),
    ).toBeInTheDocument();
  });

  it('shows the message when the API answers 403 module_disabled', async () => {
    stubApi({ enabled: true, read_only: false });
    render(<App />);
    await screen.findByRole('link', { name: /Recipes/ });
    vi.stubGlobal(
      'fetch',
      vi
        .fn()
        .mockResolvedValue(
          json({ code: 'module_disabled', status: 403, title: 'Module disabled', type: 'x' }, 403),
        ),
    );
    await expect(
      apiFetch('/v1/workspaces/x/containers', { method: 'POST', body: {} }),
    ).rejects.toThrow();
    expect(await screen.findByRole('alert')).toHaveTextContent(text('ui.modules.api_disabled'));
  });

  it('shows the read-only message when the API answers 403 module_read_only', async () => {
    stubApi({ enabled: true, read_only: false });
    render(<App />);
    await screen.findByRole('link', { name: /Recipes/ });
    vi.stubGlobal(
      'fetch',
      vi
        .fn()
        .mockResolvedValue(
          json({ code: 'module_read_only', status: 403, title: 'Read only', type: 'x' }, 403),
        ),
    );
    await expect(apiFetch('/v1/x', { method: 'POST', body: {} })).rejects.toThrow();
    expect(await screen.findByRole('alert')).toHaveTextContent(text('ui.modules.api_read_only'));
  });
});

describe('write button', () => {
  it('is aria-disabled with a text reason and ignores clicks when read only', async () => {
    stubApi({ enabled: true, read_only: true });
    const onClick = vi.fn();
    render(<App />);
    // Without a provider the button is enabled; inside App it follows the module state.
    cleanup();
    const { ModulesProvider } = await import('./ModulesProvider');
    const { WorkspaceProvider } = await import('../workspace/WorkspaceProvider');
    render(
      <WorkspaceProvider>
        <ModulesProvider>
          <ModuleWriteButton moduleId="core" onClick={onClick}>
            Save
          </ModuleWriteButton>
        </ModulesProvider>
      </WorkspaceProvider>,
    );
    const reason = text('ui.modules.read_only_reason', { module: 'Core' });
    expect(await screen.findByText(reason)).toBeInTheDocument();
    const button = screen.getByRole('button', { name: 'Save' });
    expect(button).toHaveAttribute('aria-disabled', 'true');
    fireEvent.click(button);
    expect(onClick).not.toHaveBeenCalled();
  });
});
