export function isProvidersFilterActive(providers: string[], activeProviders: string[]) {
  return activeProviders.length !== providers.length;
}
