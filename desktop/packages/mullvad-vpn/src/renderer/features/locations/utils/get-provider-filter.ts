import type { IRelayLocationRelayRedux } from '../../../redux/settings/reducers';

export function getProviderFilter(
  providers?: string[],
): ((relay: IRelayLocationRelayRedux) => boolean) | undefined {
  const filterActive = providers !== undefined && providers.length > 0;
  return filterActive ? (relay) => providers.includes(relay.provider) : undefined;
}
