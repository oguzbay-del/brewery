import type { ReactNode } from 'react';
import { useId } from 'react';
import './StateView.css';

export interface StateViewProps {
  kind: 'empty' | 'loading' | 'error';
  title: string;
  description?: string;
  action?: ReactNode;
}

export function StateView({ kind, title, description, action }: StateViewProps) {
  const titleId = useId();

  return (
    <section
      className="kit-state-view"
      data-kind={kind}
      aria-labelledby={titleId}
      aria-busy={kind === 'loading'}
    >
      <div role="status" aria-live="polite">
        <svg
          aria-hidden="true"
          focusable="false"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          {kind === 'empty' ? (
            <path d="M4 5h16v14H4zM4 14h5l2 2h2l2-2h5" />
          ) : kind === 'loading' ? (
            <path d="M12 3a9 9 0 1 0 9 9" />
          ) : (
            <path d="M12 3 2.5 20h19zM12 10v4.5M12 17.5v.5" />
          )}
        </svg>
        <h2 id={titleId}>{title}</h2>
        {description && <p>{description}</p>}
      </div>
      {action && <div>{action}</div>}
    </section>
  );
}
