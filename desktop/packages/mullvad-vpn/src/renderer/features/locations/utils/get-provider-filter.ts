import type { IRelayLocationRelayRedux } from '../../../redux/settings/reducers';

export function getProviderFilter(
  providers?: string[],
): ((relay: IRelayLocationRelayRedux) => boolean) | undefined {
  return providers === undefined || providers.length === 0
    ? undefined
    : (relay) => providers.includes(relay.provider);
}
