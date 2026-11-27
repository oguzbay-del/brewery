export type BillingProblem = 'billing_unavailable' | 'already_subscribed';

export function billingProblem(status: number, code: string): BillingProblem | null {
  if (status === 503 && code === 'billing_unavailable') return 'billing_unavailable';
  if (status === 409 && code === 'already_subscribed') return 'already_subscribed';
  return null;
}
