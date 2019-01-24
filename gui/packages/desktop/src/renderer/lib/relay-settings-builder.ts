import {
  RelayLocation,
  RelayProtocol,
  RelaySettingsUpdate,
  RelaySettingsNormalUpdate,
  OpenVpnConstraints,
} from '../../shared/daemon-rpc-types';

type LocationBuilder<Self> = {
  country: (country: string) => Self;
  city: (country: string, city: string) => Self;
  hostname: (country: string, city: string, hostname: string) => Self;
  any: () => Self;
  fromRaw: (location: 'any' | RelayLocation) => Self;
};

interface ExactOrAny<T, Self> {
  exact(value: T): Self;
  any(): Self;
}

interface OpenVPNConfigurator {
  port: ExactOrAny<number, OpenVPNConfigurator>;
  protocol: ExactOrAny<RelayProtocol, OpenVPNConfigurator>;
}

interface TunnelBuilder {
  openvpn(
    configurator: (openVpnConfigurator: OpenVPNConfigurator) => void,
  ): NormalRelaySettingsBuilder;
  any(): NormalRelaySettingsBuilder;
}

class NormalRelaySettingsBuilder {
  _payload: RelaySettingsNormalUpdate = {};

  build(): RelaySettingsUpdate {
    return {
      normal: this._payload,
    };
  }

  get location(): LocationBuilder<NormalRelaySettingsBuilder> {
    return {
      country: (country: string) => {
        this._payload.location = { only: { country } };
        return this;
      },
      city: (country: string, city: string) => {
        this._payload.location = { only: { city: [country, city] } };
        return this;
      },
      hostname: (country: string, city: string, hostname: string) => {
        this._payload.location = { only: { hostname: [country, city, hostname] } };
        return this;
      },
      any: () => {
        this._payload.location = 'any';
        return this;
      },
      fromRaw: function(location: 'any' | RelayLocation) {
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

  get tunnel(): TunnelBuilder {
    const updateOpenvpn = (next: Partial<OpenVpnConstraints>) => {
      const tunnel = this._payload.tunnel;
      if (typeof tunnel === 'string' || typeof tunnel === 'undefined') {
        this._payload.tunnel = {
          only: {
            openvpn: next,
          },
        };
      } else if (typeof tunnel === 'object') {
        const prev = (tunnel.only && tunnel.only.openvpn) || {};
        this._payload.tunnel = {
          only: {
            openvpn: { ...prev, ...next },
          },
        };
      }
    };

    return {
      openvpn: (configurator: (configurator: OpenVPNConfigurator) => void) => {
        const openvpnBuilder: OpenVPNConfigurator = {
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
        this._payload.tunnel = 'any';
        return this;
      },
    };
  }
}

export default {
  normal: () => new NormalRelaySettingsBuilder(),
};
