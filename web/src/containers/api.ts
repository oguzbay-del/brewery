import type { components } from '../../../sozlesme/istemci/typescript/src/schema';
import { apiFetch, apiFetchBlob } from '../api/client';

export type Container = components['schemas']['Container'];
export type ContainerList = components['schemas']['ContainerList'];
export type ContainerDetail = components['schemas']['ContainerDetail'];
export type ContainerEvent = components['schemas']['ContainerEvent'];
export type PackageType = components['schemas']['PackageType'];
export type PaperSize = 'a4' | 'letter';

export const CONTAINER_STATUSES = [
  'clean',
  'filled',
  'out',
  'dirty',
  'maintenance',
  'retired',
] as const;

const base = (workspaceId: string): string => `/v1/workspaces/${encodeURIComponent(workspaceId)}`;

export function listContainers(
  workspaceId: string,
  query: { status?: string; cursor?: string | null },
  signal?: AbortSignal,
): Promise<ContainerList> {
  const params = new URLSearchParams();
  if (query.status) params.set('status', query.status);
  if (query.cursor) params.set('cursor', query.cursor);
  const suffix = params.size > 0 ? `?${params.toString()}` : '';
  return apiFetch<ContainerList>(`${base(workspaceId)}/containers${suffix}`, { signal });
}

export function getContainer(
  workspaceId: string,
  id: string,
  signal?: AbortSignal,
): Promise<ContainerDetail> {
  return apiFetch<ContainerDetail>(`${base(workspaceId)}/containers/${encodeURIComponent(id)}`, {
    signal,
  });
}

export function listPackageTypes(
  workspaceId: string,
  signal?: AbortSignal,
): Promise<{ items: PackageType[] }> {
  return apiFetch(`${base(workspaceId)}/package-types`, { signal });
}

export function createContainers(
  workspaceId: string,
  body: components['schemas']['CreateContainers'],
): Promise<components['schemas']['CreatedContainers']> {
  return apiFetch(`${base(workspaceId)}/containers`, {
    method: 'POST',
    body,
    idempotencyKey: crypto.randomUUID(),
  });
}

export function retireContainer(
  workspaceId: string,
  containerId: string,
  note: string,
): Promise<Container> {
  return apiFetch(`${base(workspaceId)}/containers/${encodeURIComponent(containerId)}/retire`, {
    method: 'POST',
    body: note ? { note } : {},
  });
}

export function printLabels(
  workspaceId: string,
  containerIds: string[],
  paper: PaperSize,
): Promise<Blob> {
  return apiFetchBlob(`${base(workspaceId)}/labels?paper=${paper}`, {
    method: 'POST',
    body: { targets: containerIds.map((id) => ({ kind: 'container', id })) },
  });
}
