import { useState } from 'react';
import { createRoot } from 'react-dom/client';
import { AlarmMark, Callout, StateView, StatusBadge, SummaryCard } from '../index';
import type { CalloutTone, StateViewProps, StatusTone } from '../index';
import { galleryText as text } from './catalog';
import '../../styles/tokens.css';
import './gallery.css';

type Theme = (typeof text.themes)[number]['value'];

function Gallery() {
  const [theme, setTheme] = useState<Theme>('light');

  return (
    <main className="kit-gallery">
      <header className="kit-gallery-header">
        <h1>{text.title}</h1>
        <label>
          {text.theme}
          <select
            value={theme}
            onChange={(event) => {
              const next = text.themes.find((item) => item.value === event.target.value);
              if (next) {
                setTheme(next.value);
                document.documentElement.dataset.theme = next.value;
              }
            }}
          >
            {text.themes.map((item) => (
              <option key={item.value} value={item.value}>
                {item.label}
              </option>
            ))}
          </select>
        </label>
      </header>
      <div className="kit-gallery-grid">
        {text.cases.map((item) => {
          const tone: CalloutTone = item.id === 'error' ? 'error' : 'info';
          const status: StatusTone = item.id === 'error' ? 'error' : 'neutral';
          const kind: StateViewProps['kind'] =
            item.id === 'error' ? 'error' : item.id === 'normal' ? 'loading' : 'empty';
          const action = item.id === 'disabled' ? <button disabled>{text.retry}</button> : undefined;
          const stateTitle =
            kind === 'loading' ? text.loading : kind === 'empty' ? text.empty : item.title;
          return (
            <section key={item.id} className="kit-gallery-case" data-case={item.id}>
              <h2>{item.label}</h2>
              <Callout tone={tone} title={item.title} action={action}>
                {item.detail}
              </Callout>
              <SummaryCard
                label={item.title}
                value={text.value}
                unit={text.unit}
                description={item.detail}
              />
              <StateView
                kind={kind}
                title={stateTitle}
                description={item.detail}
                action={action}
              />
              <StatusBadge tone={status} label={item.title} />
              <div className="kit-gallery-alarms">
                <AlarmMark level={1} label={text.alarms[0]} />
                <AlarmMark level={2} label={text.alarms[1]} />
                <AlarmMark level={3} label={item.id === 'long' ? item.title : text.alarms[2]} />
              </div>
            </section>
          );
        })}
      </div>
    </main>
  );
}

const root = document.getElementById('root');
if (!root) throw new Error('Missing gallery root');
createRoot(root).render(<Gallery />);
