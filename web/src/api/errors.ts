import type { components } from '../../../sozlesme/istemci/typescript/src/schema';

export type ApiProblem = components['schemas']['ProblemDetails'];
export type ApiErrorCode = ApiProblem['code'];
export type ApiFieldIssue = components['schemas']['FieldIssue'];

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function readProblem(value: unknown): ApiProblem | null {
  if (!isRecord(value) || typeof value.code !== 'string') return null;
  return value as unknown as ApiProblem;
}

function readConflictingFields(current: Record<string, unknown>): string[] {
  const value = current.conflicting_fields;
  return Array.isArray(value)
    ? value.filter((field): field is string => typeof field === 'string')
    : [];
}

export class ApiError extends Error {
  readonly status: number;
  readonly code: ApiErrorCode;
  readonly requestId: string | null;
  readonly errors: ApiFieldIssue[];
  readonly problem: ApiProblem | null;

  constructor(
    status: number,
    problem: ApiProblem | null,
    fallbackRequestId?: string | null,
  ) {
    const code = problem?.code ?? ('internal' as ApiErrorCode);
    super(`HTTP ${status} (${code})`);
    this.name = 'ApiError';
    this.status = status;
    this.code = code;
    this.requestId = problem?.request_id ?? fallbackRequestId ?? null;
    this.errors = problem?.errors ?? [];
    this.problem = problem;
  }
}

export class StaleVersionError extends ApiError {
  readonly current: Record<string, unknown> & { conflicting_fields: string[] };

  constructor(
    status: number,
    problem: ApiProblem | null,
    fallbackRequestId?: string | null,
  ) {
    super(status, problem, fallbackRequestId);
    this.name = 'StaleVersionError';
    const raw = problem?.current;
    const current = isRecord(raw) ? raw : {};
    this.current = {
      ...current,
      conflicting_fields: readConflictingFields(current),
    };
  }
}

export async function apiErrorFromResponse(
  response: Response,
): Promise<ApiError> {
  const value: unknown = await response.clone().json().catch(() => null);
  const problem = readProblem(value);
  const requestId = response.headers.get('x-request-id');
  if (response.status === 412) {
    return new StaleVersionError(response.status, problem, requestId);
  }
  return new ApiError(response.status, problem, requestId);
}
