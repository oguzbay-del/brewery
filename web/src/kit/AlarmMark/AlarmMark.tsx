import './AlarmMark.css';

export interface AlarmMarkProps {
  level: 1 | 2 | 3;
  label: string;
  acknowledged?: boolean;
}

export function AlarmMark({ level, label, acknowledged = true }: AlarmMarkProps) {
  return (
    <span className="kit-alarm" data-level={level} data-acknowledged={acknowledged}>
      <span className="kit-alarm-shape" aria-hidden="true">
        <span>{level}</span>
      </span>
      <span>{label}</span>
    </span>
  );
}
