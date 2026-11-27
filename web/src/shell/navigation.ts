export const ROUTES = ['/', '/recipes', '/batches', '/settings'] as const;
export type AppRoute = (typeof ROUTES)[number];

/**
 * The module that each screen needs. The ids match `x-module` in the OpenAPI file.
 * `null` means the screen needs no module (Home, and Settings where modules are managed).
 */
export const ROUTE_MODULES: Record<AppRoute, string | null> = {
  '/': null,
  '/recipes': 'core',
  '/batches': 'core',
  '/settings': null,
};
