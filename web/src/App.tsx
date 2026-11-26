import { useText } from './i18n/useText';
import { AppShell, type ShellLabels } from './shell/AppShell';
import { RouteContent } from './shell/routes';
import { ThemeProvider } from './shell/ThemeProvider';
import './shell/shell.css';

export default function App() {
  const labels: ShellLabels = {
    appName: useText('ui.shell.app_name'),
    skipToContent: useText('ui.shell.skip_to_content'),
    navigation: useText('ui.shell.navigation'),
    home: useText('ui.nav.home'),
    recipes: useText('ui.nav.recipes'),
    batches: useText('ui.nav.batches'),
    settings: useText('ui.nav.settings'),
    theme: useText('ui.theme.label'),
    light: useText('ui.theme.light'),
    dark: useText('ui.theme.dark'),
    highContrast: useText('ui.theme.high_contrast'),
  };

  return (
    <ThemeProvider>
      <AppShell labels={labels}>
        <RouteContent labels={labels} />
      </AppShell>
    </ThemeProvider>
  );
}
