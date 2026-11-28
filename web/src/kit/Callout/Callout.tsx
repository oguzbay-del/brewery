import type { ReactNode } from 'react';

export type CalloutTone = 'info' | 'success' | 'warning' | 'error';

export interface CalloutProps {
  tone: CalloutTone;
  title: string;
  children?: ReactNode;
  action?: ReactNode;
}

function ToneIcon({ tone }: { tone: CalloutTone }) {
  return (
    <svg
      aria-hidden="true"
      focusable="false"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="kit-callout-icon"
    >
      {tone === 'warning' ? (
        <>
          <path d="M12 3 2.5 20h19z" />
          <path d="M12 10v4.5M12 17.5v.5" />
        </>
      ) : (
        <>
          <circle cx="12" cy="12" r="9" />
          {tone === 'success' ? (
            <path d="m7.5 12.5 3 3 6-6.5" />
          ) : tone === 'error' ? (
            <path d="m9 9 6 6m0-6-6 6" />
          ) : (
            <path d="M12 11v6M12 7.5v.5" />
          )}
        </>
      )}
    </svg>
  );
}

export function Callout({ tone, title, children, action }: CalloutProps) {
  return (
    <div className="kit-callout" data-tone={tone}>
      <div className="kit-callout-message" role="status" aria-live="polite">
        <ToneIcon tone={tone} />
        <div className="kit-callout-text">
          <strong>{title}</strong>
          {children && <div>{children}</div>}
        </div>
      </div>
      {action && <div className="kit-callout-action">{action}</div>}
    </div>
  );
}
