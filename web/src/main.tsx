import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';
import type { ShellLabels } from './shell/AppShell';
import './styles/tokens.css';
import './styles/global.css';

const root = document.getElementById('root');
if (!root) throw new Error('Missing #root element');

const data = root.dataset;
const labels: ShellLabels = {
  appName: data.appName || document.title,
  skipToContent: data.skipToContent || '',
  navigation: data.navigation || '',
  home: data.home || '',
  recipes: data.recipes || '',
  batches: data.batches || '',
  settings: data.settings || '',
  theme: data.theme || '',
  light: data.light || '',
  dark: data.dark || '',
  highContrast: data.highContrast || '',
};

createRoot(root).render(
  <StrictMode>
    <App labels={labels} />
  </StrictMode>,
);
