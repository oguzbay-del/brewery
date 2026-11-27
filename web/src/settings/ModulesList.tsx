import { ApiError } from '../api/errors';
import { resolveLocale, translate } from '../i18n/catalog';
import { useText } from '../i18n/useText';
import { useModules } from '../modules/useModules';
import { useWorkspace } from '../workspace/useWorkspace';

function ErrorMessage({ error }: { error: unknown }) {
  const generic = useText('ui.modules.error');
  const signIn = useText('ui.modules.sign_in_required');
  const notMember = useText('ui.modules.not_member');
  const requestLabel = useText('ui.modules.error_request_id');
  const problem = error instanceof ApiError ? error : null;
  let message = generic;
  if (problem?.status === 401) message = signIn;
  if (problem?.status === 404) message = notMember;
  return (
    <p role="alert">
      {message}
      {problem?.requestId ? ` — ${requestLabel}: ${problem.requestId}` : ''}
    </p>
  );
}

function ModulesForWorkspace({ workspaceId }: { workspaceId: string }) {
  const result = useModules(workspaceId);
  const loading = useText('ui.modules.loading');
  const on = useText('ui.modules.on');
  const off = useText('ui.modules.off');
  const readOnly = useText('ui.modules.read_only');

  if (result.status === 'idle' || result.status === 'loading') {
    return <p role="status">{loading}</p>;
  }
  if (result.status === 'error') return <ErrorMessage error={result.error} />;

  return (
    <ul>
      {result.modules.map((module) => {
        const key = `ui.module.${module.id}`;
        const name = translate(key, resolveLocale());
        return (
          <li key={module.id}>
            <span>{name === key ? module.id : name}</span>
            {' — '}
            <span>{module.read_only ? readOnly : module.enabled ? on : off}</span>
          </li>
        );
      })}
    </ul>
  );
}

export function ModulesList() {
  const workspace = useWorkspace();
  const heading = useText('ui.modules.title');
  const loading = useText('ui.modules.loading');
  const empty = useText('ui.modules.empty');

  return (
    <section aria-labelledby="modules-heading">
      <h2 id="modules-heading">{heading}</h2>
      {workspace.status === 'loading' && <p role="status">{loading}</p>}
      {workspace.status === 'empty' && <p>{empty}</p>}
      {workspace.status === 'error' && <ErrorMessage error={workspace.error} />}
      {workspace.status === 'ready' && <ModulesForWorkspace workspaceId={workspace.workspaceId} />}
    </section>
  );
}
