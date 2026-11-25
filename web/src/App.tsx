import { useText } from './i18n/useText';
import { AppShell, type ShellLabels } from './shell/AppShell';
import { RouteContent } from './shell/routes';
import { ThemeProvider } from './shell/ThemeProvider';
import './shell/shell.css';

export default function App() {
  const labels: ShellLabels = {
    appName: useText('ui.appName'),
    skipToContent: useText('ui.skipToContent'),
    navigation: useText('ui.navigation'),
    home: useText('ui.home'),
    recipes: useText('ui.recipes'),
    batches: useText('ui.batches'),
    settings: useText('ui.settings'),
    theme: useText('ui.theme'),
    light: useText('ui.light'),
    dark: useText('ui.dark'),
    highContrast: useText('ui.highContrast'),
  };

  return (
    <ThemeProvider>
      <AppShell labels={labels}>
        <RouteContent labels={labels} />
      </AppShell>
    </ThemeProvider>
  );
}
