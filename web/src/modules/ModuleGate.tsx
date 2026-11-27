import { useId, type MouseEvent, type ReactNode } from 'react';
import { resolveLocale, translate } from '../i18n/catalog';
import { useText } from '../i18n/useText';
import { useModuleAccess, useModuleNotice } from './ModulesProvider';

/** Display name of a module id from the text catalog; falls back to the id. */
export function useModuleName(moduleId: string): string {
  // Reading the locale through useText keeps the name in step with locale changes.
  useText('ui.modules.title');
  const key = `ui.module.${moduleId}`;
  const name = translate(key, resolveLocale());
  return name === key ? moduleId : name;
}

function UpgradeMessage({ moduleId }: { moduleId: string }) {
  const name = useModuleName(moduleId);
  const title = useText('ui.modules.upgrade_title', { module: name });
  const body = useText('ui.modules.upgrade_body', { module: name });
  const link = useText('ui.modules.upgrade_link');
  return (
    <section className="module-message" aria-labelledby="module-upgrade-heading">
      <h2 id="module-upgrade-heading">{title}</h2>
      <p>{body}</p>
      <a href="#/settings">{link}</a>
    </section>
  );
}

function ReadOnlyBanner({ moduleId }: { moduleId: string }) {
  const name = useModuleName(moduleId);
  const text = useText('ui.modules.read_only_banner', { module: name });
  return (
    <p className="module-banner" role="status">
      {text}
    </p>
  );
}

/** Shows the upgrade message for an off module and a banner for a read-only module. */
export function ModuleGate({
  moduleId,
  children,
}: {
  moduleId: string | null;
  children: ReactNode;
}) {
  const access = useModuleAccess(moduleId);
  const loading = useText('ui.modules.loading');
  if (moduleId === null) return <>{children}</>;
  if (access === 'loading') return <p role="status">{loading}</p>;
  if (access === 'off') return <UpgradeMessage moduleId={moduleId} />;
  return (
    <>
      {access === 'read_only' && <ReadOnlyBanner moduleId={moduleId} />}
      {children}
    </>
  );
}

interface ModuleWriteButtonProps {
  moduleId: string;
  onClick?: () => void;
  children: ReactNode;
}

/**
 * A button for a write action. When the module is read only, it is `aria-disabled` and a visible
 * text gives the reason; the click does nothing.
 */
export function ModuleWriteButton({ moduleId, onClick, children }: ModuleWriteButtonProps) {
  const access = useModuleAccess(moduleId);
  const name = useModuleName(moduleId);
  const reason = useText('ui.modules.read_only_reason', { module: name });
  const reasonId = useId();
  const blocked = access === 'read_only' || access === 'off';
  const handleClick = (event: MouseEvent<HTMLButtonElement>): void => {
    if (blocked) {
      event.preventDefault();
      return;
    }
    onClick?.();
  };
  return (
    <>
      <button
        type="button"
        aria-disabled={blocked ? 'true' : undefined}
        aria-describedby={blocked ? reasonId : undefined}
        onClick={handleClick}
      >
        {children}
      </button>
      {blocked && (
        <span id={reasonId} className="module-reason">
          {reason}
        </span>
      )}
    </>
  );
}

/** Alert for a 403 `module_disabled` or `module_read_only` answer to a write. */
export function ModuleNotice() {
  const { notice, dismiss } = useModuleNotice();
  const disabled = useText('ui.modules.api_disabled');
  const readOnly = useText('ui.modules.api_read_only');
  const close = useText('ui.modules.dismiss');
  if (!notice) return null;
  return (
    <div className="module-notice" role="alert">
      <span>{notice === 'disabled' ? disabled : readOnly}</span>{' '}
      <button type="button" onClick={dismiss}>
        {close}
      </button>
    </div>
  );
}
