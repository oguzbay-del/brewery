import { useCallback, useEffect, useId, useState } from 'react';
import { ApiError } from '../api/errors';
import { resolveLocale } from '../i18n/catalog';
import { useText, useTranslate } from '../i18n/useText';
import { ModuleWriteButton } from '../modules/ModuleGate';
import { useWorkspace } from '../workspace/useWorkspace';
import {
  CONTAINER_STATUSES,
  createContainers,
  getContainer,
  listContainers,
  listPackageTypes,
  printLabels,
  retireContainer,
  type Container,
  type ContainerDetail,
  type ContainerList,
  type PackageType,
  type PaperSize,
} from './api';
import './containers.css';

const MODULE_ID = 'production';

function isForbidden(error: unknown): boolean {
  return error instanceof ApiError && error.status === 403 && error.code === 'forbidden';
}

function formatTime(value: string): string {
  return new Date(value).toLocaleString(resolveLocale(), {
    dateStyle: 'medium',
    timeStyle: 'short',
  });
}

function StatusBadge({ status }: { status: string }) {
  const t = useTranslate();
  return (
    <span className={`containers__badge containers__badge--${status}`}>
      {t(`ui.containers.status_${status}`)}
    </span>
  );
}

interface Permissions {
  /** Set when the API answered 403 `forbidden` for a manage action. Then the screen hides them. */
  manageDenied: boolean;
  onForbidden: () => void;
}

function EventRow({ event }: { event: ContainerDetail['events'][number] }) {
  const t = useTranslate();
  const label = t(`ui.containers.event_${event.event_type}`);
  const change =
    event.old_status || event.new_status
      ? ` (${event.old_status ? t(`ui.containers.status_${event.old_status}`) : '–'} → ${
          event.new_status ? t(`ui.containers.status_${event.new_status}`) : '–'
        })`
      : '';
  return (
    <tr>
      <td>
        {label}
        {change}
      </td>
      <td>{event.note ?? ''}</td>
      <td>{formatTime(event.occurred_at)}</td>
    </tr>
  );
}

function RetireForm({
  workspaceId,
  container,
  permissions,
  onDone,
}: {
  workspaceId: string;
  container: Container;
  permissions: Permissions;
  onDone: () => void;
}) {
  const noteLabel = useText('ui.containers.retire_note');
  const retireLabel = useText('ui.containers.retire');
  const failed = useText('ui.containers.action_error');
  const noteId = useId();
  const [note, setNote] = useState('');
  const [error, setError] = useState(false);

  if (permissions.manageDenied || container.status === 'retired') return null;

  const retire = (): void => {
    setError(false);
    retireContainer(workspaceId, container.id, note.trim()).then(onDone, (cause: unknown) => {
      if (isForbidden(cause)) permissions.onForbidden();
      else setError(true);
    });
  };

  return (
    <div className="containers__retire">
      <label htmlFor={noteId}>{noteLabel}</label>
      <input id={noteId} value={note} maxLength={2000} onChange={(e) => setNote(e.target.value)} />
      <ModuleWriteButton moduleId={MODULE_ID} onClick={retire}>
        {retireLabel}
      </ModuleWriteButton>
      {error && <p role="alert">{failed}</p>}
    </div>
  );
}

function Detail({
  workspaceId,
  id,
  permissions,
  onClose,
  onChanged,
}: {
  workspaceId: string;
  id: string;
  permissions: Permissions;
  onClose: () => void;
  onChanged: () => void;
}) {
  const t = useTranslate();
  const title = useText('ui.containers.detail');
  const closeLabel = useText('ui.containers.close_detail');
  const loading = useText('ui.containers.loading');
  const failed = useText('ui.containers.error');
  const eventsTitle = useText('ui.containers.events');
  const eventsEmpty = useText('ui.containers.events_empty');
  const colEvent = useText('ui.containers.col_event');
  const colNote = useText('ui.containers.col_note');
  const colWhen = useText('ui.containers.col_when');
  const statusLabel = useText('ui.containers.status');
  const typeLabel = useText('ui.containers.package_type');
  const [state, setState] = useState<
    | { id: string; status: 'ready'; detail: ContainerDetail }
    | { id: string; status: 'error' }
    | null
  >(null);
  const [reload, setReload] = useState(0);

  useEffect(() => {
    const controller = new AbortController();
    getContainer(workspaceId, id, controller.signal).then(
      (detail) => {
        if (!controller.signal.aborted) setState({ id, status: 'ready', detail });
      },
      () => {
        if (!controller.signal.aborted) setState({ id, status: 'error' });
      },
    );
    return () => controller.abort();
  }, [workspaceId, id, reload]);

  const current = state?.id === id ? state : null;

  return (
    <section className="containers__detail" aria-labelledby="containers-detail-heading">
      <div className="containers__row">
        <h2 id="containers-detail-heading">{title}</h2>
        <button type="button" onClick={onClose}>
          {closeLabel}
        </button>
      </div>
      {!current && <p role="status">{loading}</p>}
      {current?.status === 'error' && <p role="alert">{failed}</p>}
      {current?.status === 'ready' && (
        <>
          <dl className="containers__facts">
            <dt>{t('ui.containers.serial')}</dt>
            <dd className="containers__mono">{current.detail.container.serial_number}</dd>
            <dt>{typeLabel}</dt>
            <dd>{current.detail.container.package_type_name}</dd>
            <dt>{statusLabel}</dt>
            <dd>
              <StatusBadge status={current.detail.container.status} />
            </dd>
          </dl>
          <h3>{eventsTitle}</h3>
          {current.detail.events.length === 0 ? (
            <p>{eventsEmpty}</p>
          ) : (
            <div className="containers__scroll">
              <table>
                <thead>
                  <tr>
                    <th scope="col">{colEvent}</th>
                    <th scope="col">{colNote}</th>
                    <th scope="col">{colWhen}</th>
                  </tr>
                </thead>
                <tbody>
                  {current.detail.events.map((event) => (
                    <EventRow key={event.id} event={event} />
                  ))}
                </tbody>
              </table>
            </div>
          )}
          <RetireForm
            workspaceId={workspaceId}
            container={current.detail.container}
            permissions={permissions}
            onDone={() => {
              setReload((value) => value + 1);
              onChanged();
            }}
          />
        </>
      )}
    </section>
  );
}

function CreateForm({
  workspaceId,
  permissions,
  onCreated,
}: {
  workspaceId: string;
  permissions: Permissions;
  onCreated: () => void;
}) {
  const title = useText('ui.containers.create_title');
  const typeLabel = useText('ui.containers.package_type');
  const prefixLabel = useText('ui.containers.prefix');
  const countLabel = useText('ui.containers.count');
  const createLabel = useText('ui.containers.create');
  const noTypes = useText('ui.containers.package_types_empty');
  const failed = useText('ui.containers.action_error');
  const typeId = useId();
  const prefixId = useId();
  const countId = useId();
  const [types, setTypes] = useState<PackageType[] | null>(null);
  const [packageTypeId, setPackageTypeId] = useState('');
  const [prefix, setPrefix] = useState('KEG-');
  const [count, setCount] = useState('1');
  const [error, setError] = useState(false);

  useEffect(() => {
    if (permissions.manageDenied) return;
    const controller = new AbortController();
    listPackageTypes(workspaceId, controller.signal).then(
      ({ items }) => {
        if (controller.signal.aborted) return;
        setTypes(items);
        setPackageTypeId((current) => current || items[0]?.id || '');
      },
      () => {
        if (!controller.signal.aborted) setTypes([]);
      },
    );
    return () => controller.abort();
  }, [workspaceId, permissions.manageDenied]);

  if (permissions.manageDenied) return null;

  const create = (): void => {
    setError(false);
    const parsed = Number.parseInt(count, 10);
    if (!packageTypeId || !prefix || !Number.isInteger(parsed) || parsed < 1) {
      setError(true);
      return;
    }
    createContainers(workspaceId, { package_type_id: packageTypeId, prefix, count: parsed }).then(
      onCreated,
      (cause: unknown) => {
        if (isForbidden(cause)) permissions.onForbidden();
        else setError(true);
      },
    );
  };

  return (
    <section className="containers__create" aria-labelledby="containers-create-heading">
      <h2 id="containers-create-heading">{title}</h2>
      {types && types.length === 0 && <p>{noTypes}</p>}
      <div className="containers__form">
        <label htmlFor={typeId}>{typeLabel}</label>
        <select
          id={typeId}
          value={packageTypeId}
          onChange={(e) => setPackageTypeId(e.target.value)}
        >
          {(types ?? []).map((type) => (
            <option key={type.id} value={type.id}>
              {type.name}
            </option>
          ))}
        </select>
        <label htmlFor={prefixId}>{prefixLabel}</label>
        <input
          id={prefixId}
          value={prefix}
          maxLength={90}
          onChange={(e) => setPrefix(e.target.value)}
        />
        <label htmlFor={countId}>{countLabel}</label>
        <input
          id={countId}
          type="number"
          min={1}
          max={500}
          value={count}
          onChange={(e) => setCount(e.target.value)}
        />
        <ModuleWriteButton moduleId={MODULE_ID} onClick={create}>
          {createLabel}
        </ModuleWriteButton>
      </div>
      {error && <p role="alert">{failed}</p>}
    </section>
  );
}

function downloadBlob(blob: Blob, name: string): void {
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = name;
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
}

function ContainersForWorkspace({ workspaceId }: { workspaceId: string }) {
  const t = useTranslate();
  const loading = useText('ui.containers.loading');
  const failed = useText('ui.containers.error');
  const empty = useText('ui.containers.empty');
  const countsTitle = useText('ui.containers.status_counts');
  const allLabel = useText('ui.containers.filter_all');
  const moreLabel = useText('ui.containers.load_more');
  const serialLabel = useText('ui.containers.serial');
  const statusLabel = useText('ui.containers.status');
  const typeLabel = useText('ui.containers.package_type');
  const paperLabel = useText('ui.containers.paper');
  const printLabel = useText('ui.containers.print_labels');
  const printNone = useText('ui.containers.print_none');
  const printFailed = useText('ui.containers.print_error');
  const a4 = useText('ui.containers.paper_a4');
  const letter = useText('ui.containers.paper_letter');
  const paperId = useId();

  const [filter, setFilter] = useState<string | null>(null);
  const [page, setPage] = useState<
    { key: string; state: 'ready'; data: ContainerList } | { key: string; state: 'error' } | null
  >(null);
  const [more, setMore] = useState<Container[]>([]);
  const [moreCursor, setMoreCursor] = useState<string | null | undefined>(undefined);
  const [reload, setReload] = useState(0);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [detailId, setDetailId] = useState<string | null>(null);
  const [paper, setPaper] = useState<PaperSize>('a4');
  const [printMessage, setPrintMessage] = useState<string | null>(null);
  const [manageDenied, setManageDenied] = useState(false);

  const key = `${workspaceId}|${filter ?? ''}|${reload}`;

  useEffect(() => {
    const controller = new AbortController();
    listContainers(workspaceId, { status: filter ?? undefined }, controller.signal).then(
      (data) => {
        if (controller.signal.aborted) return;
        setMore([]);
        setMoreCursor(undefined);
        setPage({ key, state: 'ready', data });
      },
      () => {
        if (!controller.signal.aborted) setPage({ key, state: 'error' });
      },
    );
    return () => controller.abort();
  }, [workspaceId, filter, key]);

  const permissions: Permissions = {
    manageDenied,
    onForbidden: useCallback(() => setManageDenied(true), []),
  };

  const refresh = (): void => setReload((value) => value + 1);

  const current = page?.key === key ? page : null;
  if (!current && !page) return <p role="status">{loading}</p>;

  const data = page?.state === 'ready' ? page.data : null;
  const items = data ? [...data.items, ...more] : [];
  const nextCursor = moreCursor === undefined ? (data?.next_cursor ?? null) : moreCursor;

  const showMore = (): void => {
    if (!nextCursor) return;
    listContainers(workspaceId, { status: filter ?? undefined, cursor: nextCursor }).then(
      (next) => {
        setMore((existing) => [...existing, ...next.items]);
        setMoreCursor(next.next_cursor ?? null);
      },
      () => setPage({ key, state: 'error' }),
    );
  };

  const toggle = (id: string): void =>
    setSelected((existing) => {
      const next = new Set(existing);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });

  const print = (): void => {
    setPrintMessage(null);
    if (selected.size === 0) {
      setPrintMessage(printNone);
      return;
    }
    printLabels(workspaceId, [...selected], paper).then(
      (blob) => downloadBlob(blob, 'labels.pdf'),
      (cause: unknown) => {
        if (isForbidden(cause)) permissions.onForbidden();
        else setPrintMessage(printFailed);
      },
    );
  };

  return (
    <div className="containers">
      {!current && <p role="status">{loading}</p>}
      {page?.state === 'error' && <p role="alert">{failed}</p>}
      {data && (
        <>
          <section aria-labelledby="containers-counts-heading">
            <h2 id="containers-counts-heading" className="containers__visually-hidden">
              {countsTitle}
            </h2>
            <ul className="containers__counts">
              <li>
                <button
                  type="button"
                  aria-pressed={filter === null}
                  onClick={() => setFilter(null)}
                >
                  {allLabel}
                </button>
              </li>
              {CONTAINER_STATUSES.map((status) => {
                const count =
                  data.status_counts.find((entry) => entry.status === status)?.count ?? 0;
                return (
                  <li key={status}>
                    <button
                      type="button"
                      aria-pressed={filter === status}
                      onClick={() => setFilter(status)}
                    >
                      {t(`ui.containers.status_${status}`)} <b>{count}</b>
                    </button>
                  </li>
                );
              })}
            </ul>
          </section>

          {!manageDenied && (
            <div className="containers__row">
              <label htmlFor={paperId}>{paperLabel}</label>
              <select
                id={paperId}
                value={paper}
                onChange={(e) => setPaper(e.target.value === 'letter' ? 'letter' : 'a4')}
              >
                <option value="a4">{a4}</option>
                <option value="letter">{letter}</option>
              </select>
              <ModuleWriteButton moduleId={MODULE_ID} onClick={print}>
                {printLabel}
              </ModuleWriteButton>
              {printMessage && <p role="alert">{printMessage}</p>}
            </div>
          )}

          {items.length === 0 ? (
            <p>{empty}</p>
          ) : (
            <div className="containers__scroll">
              <table>
                <thead>
                  <tr>
                    {!manageDenied && (
                      <th scope="col">
                        <span className="containers__visually-hidden">{printLabel}</span>
                      </th>
                    )}
                    <th scope="col">{serialLabel}</th>
                    <th scope="col">{typeLabel}</th>
                    <th scope="col">{statusLabel}</th>
                  </tr>
                </thead>
                <tbody>
                  {items.map((container) => (
                    <tr key={container.id}>
                      {!manageDenied && (
                        <td>
                          <input
                            type="checkbox"
                            aria-label={t('ui.containers.select_one', {
                              serial: container.serial_number,
                            })}
                            checked={selected.has(container.id)}
                            onChange={() => toggle(container.id)}
                          />
                        </td>
                      )}
                      <td>
                        <button
                          type="button"
                          className="containers__link containers__mono"
                          aria-expanded={detailId === container.id}
                          onClick={() => setDetailId(container.id)}
                        >
                          {container.serial_number}
                        </button>
                      </td>
                      <td>{container.package_type_name}</td>
                      <td>
                        <StatusBadge status={container.status} />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
          {nextCursor && (
            <button type="button" onClick={showMore}>
              {moreLabel}
            </button>
          )}
        </>
      )}

      {detailId && (
        <Detail
          workspaceId={workspaceId}
          id={detailId}
          permissions={permissions}
          onClose={() => setDetailId(null)}
          onChanged={refresh}
        />
      )}

      <CreateForm workspaceId={workspaceId} permissions={permissions} onCreated={refresh} />
    </div>
  );
}

/** The "Fıçılar" screen: kegs with status counts, a detail view with the event log, label print. */
export function ContainersScreen() {
  const workspace = useWorkspace();
  const loading = useText('ui.containers.loading');
  const failed = useText('ui.containers.error');
  const empty = useText('ui.modules.empty');

  if (workspace.status === 'loading') return <p role="status">{loading}</p>;
  if (workspace.status === 'empty') return <p>{empty}</p>;
  if (workspace.status === 'error') return <p role="alert">{failed}</p>;
  return <ContainersForWorkspace workspaceId={workspace.workspaceId} />;
}
