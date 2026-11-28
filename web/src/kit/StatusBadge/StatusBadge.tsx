import './StatusBadge.css';

export type StatusTone = 'neutral' | 'info' | 'success' | 'warning' | 'error';

export interface StatusBadgeProps {
  tone: StatusTone;
  label: string;
}

export function StatusBadge({ tone, label }: StatusBadgeProps) {
  return (
    <span className="kit-status-badge" data-tone={tone}>
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
        {tone === 'success' ? (
          <path d="m5 12 4 4L19 6" />
        ) : tone === 'warning' ? (
          <path d="M12 3 2.5 20h19zM12 10v4.5M12 17.5v.5" />
        ) : tone === 'error' ? (
          <path d="m6 6 12 12M6 18 18 6" />
        ) : tone === 'info' ? (
          <path d="M12 11v7M12 6v.5" />
        ) : (
          <circle cx="12" cy="12" r="5" />
        )}
      </svg>
      <span>{label}</span>
    </span>
  );
}
