import { useRef, useState } from 'react';
import { createCheckout, createPortal, type BillingPlan } from '../api/billing';
import { billingProblem, type BillingProblem } from './billingErrors';

export function useBillingActions(workspaceId: string, csrfToken: string) {
  const [denied, setDenied] = useState(false);
  const [pending, setPending] = useState(false);
  const [problem, setProblem] = useState<BillingProblem | 'unknown' | null>(null);
  const busy = useRef(false);

  async function run(action: () => Promise<string>): Promise<void> {
    if (busy.current || denied || !csrfToken) return;
    busy.current = true;
    setPending(true);
    setProblem(null);
    try {
      const url = new URL(await action());
      if (url.protocol !== 'https:') throw new Error('Invalid billing URL');
      window.location.assign(url.href);
    } catch (error: unknown) {
      const kind = billingProblem(error);
      if (kind === 'forbidden') setDenied(true);
      setProblem(kind ?? 'unknown');
    } finally {
      busy.current = false;
      setPending(false);
    }
  }

  return {
    denied,
    pending,
    problem,
    checkout: (plan: BillingPlan, seats: number) =>
      run(async () => (await createCheckout(workspaceId, plan, seats, csrfToken)).checkout_url),
    portal: () => run(async () => (await createPortal(workspaceId, csrfToken)).portal_url),
  };
}
