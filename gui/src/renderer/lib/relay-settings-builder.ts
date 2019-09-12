import {
  Constraint,
  IOpenVpnConstraints,
  IWireguardConstraints,
  LiftedConstraint,
  RelayLocation,
  RelayProtocol,
  RelaySettingsNormalUpdate,
  RelaySettingsUpdate,
  TunnelProtocol,
} from '../../shared/daemon-rpc-types';

interface ILocationBuilder<Self> {
  country: (country: string) => Self;
  city: (country: string, city: string) => Self;
  hostname: (country: string, city: string, hostname: string) => Self;
  any: () => Self;
  fromRaw: (location: LiftedConstraint<RelayLocation>) => Self;
}

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
    return {
      country: (country: string) => {
        this.payload.location = { only: { country } };
        return this;
      },
      city: (country: string, city: string) => {
        this.payload.location = { only: { city: [country, city] } };
        return this;
      },
      hostname: (country: string, city: string, hostname: string) => {
        this.payload.location = { only: { hostname: [country, city, hostname] } };
        return this;
      },
      any: () => {
        this.payload.location = 'any';
        return this;
      },
      fromRaw(location: LiftedConstraint<RelayLocation>) {
        if (location === 'any') {
          return this.any();
        } else if ('hostname' in location) {
          const [country, city, hostname] = location.hostname;
          return this.hostname(country, city, hostname);
        } else if ('city' in location) {
          const [country, city] = location.city;
          return this.city(country, city);
        } else if ('country' in location) {
          return this.country(location.country);
        }

        throw new Error(
          'Unsupported value of RelayLocation' + (location && JSON.stringify(location)),
        );
      },
    };
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
