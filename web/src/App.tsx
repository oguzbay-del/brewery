import { AppShell, type ShellLabels } from './shell/AppShell';
import { RouteContent } from './shell/routes';
import { ThemeProvider } from './shell/ThemeProvider';
import './shell/shell.css';

function defaultLabels(): ShellLabels {
  return {
    appName: typeof document === 'undefined' ? '' : document.title,
    skipToContent: '',
    navigation: '',
    home: '',
    recipes: '',
    batches: '',
    settings: '',
    theme: '',
    light: '',
    dark: '',
    highContrast: '',
  };
}

export default function App({ labels = defaultLabels() }: { labels?: ShellLabels } = {}) {
  return (
    <ThemeProvider>
      <AppShell labels={labels}>
        <RouteContent labels={labels} />
      </AppShell>
    </ThemeProvider>
  );
}
