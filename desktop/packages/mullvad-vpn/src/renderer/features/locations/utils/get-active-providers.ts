export function getActiveProviders(providers: string[], providerConstraint: string[]) {
  let activeProviders = [];

  // Empty constraint array means that all providers are selected. No selection isn't possible.
  if (providerConstraint.length === 0) {
    activeProviders = providers;
  } else {
    activeProviders = providers.filter((provider) => providerConstraint.includes(provider));
  }

  return activeProviders;
}
