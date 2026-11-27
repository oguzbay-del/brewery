import { apiFetch } from './client';

export type BillingPlan = 'home' | 'business';

export interface BillingView {
  subscription: {
    access: string;
    access_source: string;
    trial_ends_at?: string | null;
    valid_until?: string | null;
    plan_key?: BillingPlan | null;
    seats?: number | null;
    subscriptions: {
      source: string;
      product_key: string;
      status: string;
      period_ends_at?: string | null;
    }[];
  };
  active_source?: string | null;
  web_billing_available: boolean;
}

export function fetchBilling(workspaceId: string, signal?: AbortSignal): Promise<BillingView> {
  return apiFetch<BillingView>(`/v1/billing?workspace_id=${encodeURIComponent(workspaceId)}`, {
    signal,
  });
}

export function createCheckout(
  workspaceId: string,
  plan: BillingPlan,
  seats: number,
  csrfToken: string,
) {
  return apiFetch<{ checkout_url: string }>('/v1/billing/checkout', {
    method: 'POST',
    headers: { 'X-CSRF-Token': csrfToken },
    body: { workspace_id: workspaceId, plan, seats },
  });
}

export function createPortal(workspaceId: string, csrfToken: string) {
  return apiFetch<{ portal_url: string }>('/v1/billing/portal', {
    method: 'POST',
    headers: { 'X-CSRF-Token': csrfToken },
    body: { workspace_id: workspaceId },
  });
}
