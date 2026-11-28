import type { ReactNode } from 'react';
import { useId } from 'react';
import './SummaryCard.css';

export interface SummaryCardProps {
  label: string;
  value: string;
  unit?: string;
  description?: string;
  icon?: ReactNode;
}

export function SummaryCard({ label, value, unit, description, icon }: SummaryCardProps) {
  const labelId = useId();

  return (
    <section className="kit-summary-card" aria-labelledby={labelId}>
      <div className="kit-summary-label" id={labelId}>
        {icon && <span aria-hidden="true">{icon}</span>}
        <span>{label}</span>
      </div>
      <div className="kit-summary-metric">
        <span className="kit-summary-value">{value}</span>
        {unit && <span className="kit-summary-unit">{unit}</span>}
      </div>
      {description && <p className="kit-summary-description">{description}</p>}
    </section>
  );
}
