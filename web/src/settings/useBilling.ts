import { useEffect, useState } from 'react';
import { fetchBilling, type BillingView } from '../api/billing';

type BillingState =
  | { status: 'loading' }
  | { status: 'error' }
  | { status: 'ready'; data: BillingView };

export function useBilling(workspaceId: string): BillingState {
  const [result, setResult] = useState<{
    workspaceId: string;
    state: BillingState;
  } | null>(null);

  useEffect(() => {
    const controller = new AbortController();
    fetchBilling(workspaceId, controller.signal).then(
      (data) => {
        if (!controller.signal.aborted) {
          setResult({ workspaceId, state: { status: 'ready', data } });
        }
      },
      () => {
        if (!controller.signal.aborted) {
          setResult({ workspaceId, state: { status: 'error' } });
        }
      },
    );
    return () => controller.abort();
  }, [workspaceId]);

  return result?.workspaceId === workspaceId ? result.state : { status: 'loading' };
}
