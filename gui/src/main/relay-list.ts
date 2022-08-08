import { BridgeState, IRelayList, liftConstraint, RelaySettings } from '../shared/daemon-rpc-types';
import { IpcMainEventChannel } from './ipc-event-channel';

interface RelayLists {
  relays: IRelayList;
  bridges: IRelayList;
}

export default class RelayList {
  private relays: IRelayList = { countries: [] };

  public setRelays(
    newRelayList: IRelayList,
    relaySettings: RelaySettings,
    bridgeState: BridgeState,
  ) {
    this.relays = newRelayList;

    const processedRelays = this.processRelays(newRelayList, relaySettings, bridgeState);
    IpcMainEventChannel.relays.notify?.(processedRelays);
  }

  public updateSettings(relaySettings: RelaySettings, bridgeState: BridgeState) {
    this.setRelays(this.relays, relaySettings, bridgeState);
  }

  public getProcessedRelays(relaySettings: RelaySettings, bridgeState: BridgeState) {
    return this.processRelays(this.relays, relaySettings, bridgeState);
  }

  private processRelays(
    relayList: IRelayList,
    relaySettings: RelaySettings,
    bridgeState: BridgeState,
  ): RelayLists {
    const filteredRelays = this.processRelaysForPresentation(relayList, relaySettings);
    const filteredBridges = this.processBridgesForPresentation(relayList, bridgeState);

    return { relays: filteredRelays, bridges: filteredBridges };
  }

  private processRelaysForPresentation(
    relayList: IRelayList,
    relaySettings: RelaySettings,
  ): IRelayList {
    const tunnelProtocol =
      'normal' in relaySettings ? liftConstraint(relaySettings.normal.tunnelProtocol) : undefined;

    const filteredCountries = relayList.countries
      .map((country) => ({
        ...country,
        cities: country.cities
          .map((city) => ({
            ...city,
            relays: city.relays.filter((relay) => {
              if (relay.endpointType != 'bridge') {
                switch (tunnelProtocol) {
                  case 'openvpn':
                    return relay.endpointType == 'openvpn';

                  case 'wireguard':
                    return relay.endpointType == 'wireguard';

                  case 'any': {
                    const useMultihop =
                      'normal' in relaySettings &&
                      relaySettings.normal.wireguardConstraints.useMultihop;
                    return !useMultihop || relay.endpointType == 'wireguard';
                  }
                  default:
                    return false;
                }
              } else {
                return false;
              }
            }),
          }))
          .filter((city) => city.relays.length > 0),
      }))
      .filter((country) => country.cities.length > 0);

    return {
      countries: filteredCountries,
    };
  }

  private processBridgesForPresentation(
    relayList: IRelayList,
    bridgeState: BridgeState,
  ): IRelayList {
    if (bridgeState === 'on') {
      const filteredCountries = relayList.countries
        .map((country) => ({
          ...country,
          cities: country.cities
            .map((city) => ({
              ...city,
              relays: city.relays.filter((relay) => relay.endpointType == 'bridge'),
            }))
            .filter((city) => city.relays.length > 0),
        }))
        .filter((country) => country.cities.length > 0);

      return { countries: filteredCountries };
    } else {
      return { countries: [] };
    }
  }
}
