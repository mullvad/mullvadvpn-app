import {
  BridgeState,
  IRelayList,
  IRelayListWithEndpointData,
  liftConstraint,
  RelaySettings,
} from '../shared/daemon-rpc-types';
import { IRelayListPair } from '../shared/ipc-schema';
import { IpcMainEventChannel } from './ipc-event-channel';

export default class RelayList {
  private relays: IRelayListWithEndpointData = {
    relayList: {
      countries: [],
    },
    wireguardEndpointData: {
      portRanges: [],
      udp2tcpPorts: [],
    },
  };

  public setRelays(
    newRelayList: IRelayListWithEndpointData,
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
    relayList: IRelayListWithEndpointData,
    relaySettings: RelaySettings,
    bridgeState: BridgeState,
  ): IRelayListPair {
    const filteredRelays = this.processRelaysForPresentation(relayList.relayList, relaySettings);
    const filteredBridges = this.processBridgesForPresentation(relayList.relayList, bridgeState);

    return {
      relays: filteredRelays,
      bridges: filteredBridges,
      wireguardEndpointData: relayList.wireguardEndpointData,
    };
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

    return { countries: filteredCountries };
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
