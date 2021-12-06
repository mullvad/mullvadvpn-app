import {
  Constraint,
  IOpenVpnConstraints,
  IpVersion,
  IWireguardConstraints,
  RelayLocation,
  RelayProtocol,
  RelaySettingsNormalUpdate,
  RelaySettingsUpdate,
  TunnelProtocol,
} from './daemon-rpc-types';
import makeLocationBuilder, { ILocationBuilder } from './relay-location-builder';

interface IExactOrAny<T, Self> {
  exact(value: T): Self;
  any(): Self;
}

interface IOpenVPNConfigurator {
  port: IExactOrAny<number, IOpenVPNConfigurator>;
  protocol: IExactOrAny<RelayProtocol, IOpenVPNConfigurator>;
}

interface IWireguardConfigurator {
  port: IExactOrAny<number, IWireguardConfigurator>;
  ipVersion: IExactOrAny<IpVersion, IWireguardConfigurator>;
  useMultihop: (value: boolean) => IWireguardConfigurator;
  entryLocation: IExactOrAny<RelayLocation, IWireguardConfigurator>;
}

interface ITunnelProtocolConfigurator {
  tunnelProtocol: IExactOrAny<TunnelProtocol, ITunnelProtocolConfigurator>;
}

interface ITunnelBuilder {
  openvpn(
    configurator: (openVpnConfigurator: IOpenVPNConfigurator) => void,
  ): NormalRelaySettingsBuilder;
  wireguard(
    configurator: (wireguardConfigurator: IWireguardConfigurator) => void,
  ): NormalRelaySettingsBuilder;
  tunnelProtocol(
    configurator: (tunnelProtocolConfigurator: ITunnelProtocolConfigurator) => void,
  ): NormalRelaySettingsBuilder;
}

class NormalRelaySettingsBuilder {
  private payload: RelaySettingsNormalUpdate = {};

  public build(): RelaySettingsUpdate {
    return {
      normal: this.payload,
    };
  }

  get location(): ILocationBuilder<NormalRelaySettingsBuilder> {
    return makeLocationBuilder(this, (location) => {
      this.payload.location = location;
    });
  }

  get tunnel(): ITunnelBuilder {
    const updateOpenvpn = (next: Partial<IOpenVpnConstraints>) => {
      if (this.payload.openvpnConstraints === undefined) {
        this.payload.openvpnConstraints = next;
      } else {
        const prev = this.payload.openvpnConstraints;
        this.payload.openvpnConstraints = {
          ...prev,
          ...next,
        };
      }
    };

    const updateWireguard = (next: Partial<IWireguardConstraints>) => {
      if (this.payload.wireguardConstraints === undefined) {
        this.payload.wireguardConstraints = next;
      } else {
        const prev = this.payload.wireguardConstraints;
        this.payload.wireguardConstraints = {
          ...prev,
          ...next,
        };
      }
    };

    const updateTunnelProtocol = (next?: Constraint<TunnelProtocol>) => {
      this.payload.tunnelProtocol = next;
    };

    return {
      openvpn: (configurator: (configurator: IOpenVPNConfigurator) => void) => {
        const openvpnBuilder: IOpenVPNConfigurator = {
          get port() {
            const apply = (port: Constraint<number>) => {
              updateOpenvpn({ port });
              return this;
            };
            return {
              exact: (value: number) => apply({ only: value }),
              any: () => apply('any'),
            };
          },
          get protocol() {
            const apply = (protocol: Constraint<RelayProtocol>) => {
              updateOpenvpn({ protocol });
              return this;
            };
            return {
              exact: (value: RelayProtocol) => apply({ only: value }),
              any: () => apply('any'),
            };
          },
        };

        configurator(openvpnBuilder);

        return this;
      },

      wireguard: (configurator: (configurator: IWireguardConfigurator) => void) => {
        const wireguardBuilder: IWireguardConfigurator = {
          get port() {
            const apply = (port: Constraint<number>) => {
              updateWireguard({ port });
              return this;
            };
            return {
              exact: (value: number) => apply({ only: value }),
              any: () => apply('any'),
            };
          },
          get ipVersion() {
            const apply = (ipVersion: Constraint<IpVersion>) => {
              updateWireguard({ ipVersion });
              return this;
            };
            return {
              exact: (value: IpVersion) => apply({ only: value }),
              any: () => apply('any'),
            };
          },
          get useMultihop() {
            return (useMultihop: boolean) => {
              updateWireguard({ useMultihop });
              return this;
            };
          },
          get entryLocation() {
            const apply = (entryLocation: Constraint<RelayLocation> | undefined) => {
              updateWireguard({ entryLocation });
              return this;
            };
            return {
              exact: (entryLocation: RelayLocation) => apply({ only: entryLocation }),
              any: () => apply('any'),
            };
          },
        };
        configurator(wireguardBuilder);
        return this;
      },

      tunnelProtocol: (configurator: (configurator: ITunnelProtocolConfigurator) => void) => {
        const tunnelProtocolBuilder = {
          get tunnelProtocol() {
            return {
              exact: (value: TunnelProtocol) => {
                updateTunnelProtocol({ only: value });
                return this;
              },
              any: () => {
                updateTunnelProtocol('any');
                return this;
              },
            };
          },
        };

        configurator(tunnelProtocolBuilder);
        return this;
      },
    };
  }
}

export default {
  normal: () => new NormalRelaySettingsBuilder(),
};
