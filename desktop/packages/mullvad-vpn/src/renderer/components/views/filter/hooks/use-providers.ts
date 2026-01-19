import { useNormalRelaySettings } from '../../../../lib/relay-settings-hooks';
import { useSelector } from '../../../../redux/store';
import { providersFromRelays } from '../utils';

export function useProviders(): Record<string, boolean> {
  const relaySettings = useNormalRelaySettings();
  const locations = useSelector((state) => state.settings.relayLocations);
  const providerConstraint = relaySettings?.providers ?? [];

  const providers = providersFromRelays(locations);

  // Empty containt array means that all providers are selected. No selection isn't possible.
  return Object.fromEntries(
    providers.map((provider) => [
      provider,
      providerConstraint.length === 0 || providerConstraint.includes(provider),
    ]),
  );
}
