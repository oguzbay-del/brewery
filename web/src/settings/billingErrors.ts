import { ApiError } from '../api/errors';

export type BillingProblem = 'billing_unavailable' | 'already_subscribed' | 'other_source_active';

export function billingProblem(error: unknown): BillingProblem | null {
  if (!(error instanceof ApiError)) return null;
  if (error.status === 503 && error.code === 'billing_unavailable') return 'billing_unavailable';
  if (error.status === 409) {
    if (error.errors.some((issue) => issue.code === 'already_subscribed')) {
      return 'already_subscribed';
    }
    if (error.errors.some((issue) => issue.code === 'other_source_active')) {
      return 'other_source_active';
    }
  }
  return null;
}
