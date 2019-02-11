import {
  IOpenVpnConstraints,
  RelayLocation,
  RelayProtocol,
  RelaySettingsNormalUpdate,
  RelaySettingsUpdate,
} from '../../shared/daemon-rpc-types';

interface ILocationBuilder<Self> {
  country: (country: string) => Self;
  city: (country: string, city: string) => Self;
  hostname: (country: string, city: string, hostname: string) => Self;
  any: () => Self;
  fromRaw: (location: 'any' | RelayLocation) => Self;
}

interface IExactOrAny<T, Self> {
  exact(value: T): Self;
  any(): Self;
}

interface IOpenVPNConfigurator {
  port: IExactOrAny<number, IOpenVPNConfigurator>;
  protocol: IExactOrAny<RelayProtocol, IOpenVPNConfigurator>;
}

interface ITunnelBuilder {
  openvpn(
    configurator: (openVpnConfigurator: IOpenVPNConfigurator) => void,
  ): NormalRelaySettingsBuilder;
  any(): NormalRelaySettingsBuilder;
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
      fromRaw(location: 'any' | RelayLocation) {
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
      const tunnel = this.payload.tunnel;
      if (typeof tunnel === 'string' || typeof tunnel === 'undefined') {
        this.payload.tunnel = {
          only: {
            openvpn: next,
          },
        };
      } else if (typeof tunnel === 'object') {
        const prev = tunnel.only && 'openvpn' in tunnel.only ? tunnel.only.openvpn : {};
        this.payload.tunnel = {
          only: {
            openvpn: { ...prev, ...next },
          },
        };
      }
    };

    return {
      openvpn: (configurator: (configurator: IOpenVPNConfigurator) => void) => {
        const openvpnBuilder: IOpenVPNConfigurator = {
          get port() {
            const apply = (port: 'any' | { only: number }) => {
              updateOpenvpn({ port });
              return this;
            };
            return {
              exact: (value: number) => apply({ only: value }),
              any: () => apply('any'),
            };
          },
          get protocol() {
            const apply = (protocol: 'any' | { only: RelayProtocol }) => {
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
      any: () => {
        this.payload.tunnel = 'any';
        return this;
      },
    };
  }
}

export default {
  normal: () => new NormalRelaySettingsBuilder(),
};
