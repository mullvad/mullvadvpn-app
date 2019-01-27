// @flow

import type {
  RelayLocation,
  RelayProtocol,
  RelaySettingsUpdate,
  RelaySettingsNormalUpdate,
} from '../../shared/daemon-rpc-types';

type LocationBuilder<Self> = {
  country: (country: string) => Self,
  city: (country: string, city: string) => Self,
  any: () => Self,
  fromRaw: (location: 'any' | RelayLocation) => Self,
};

type OpenVPNConfigurator<Self> = {
  port: {
    exact: (port: number) => Self,
    any: () => Self,
  },
  protocol: {
    exact: (protocol: RelayProtocol) => Self,
    any: () => Self,
  },
};

type TunnelBuilder<Self> = {
  openvpn: (configurator: (OpenVPNConfigurator<*>) => void) => Self,
};

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
        }

        if (location.hostname) {
          const [country, city, hostname] = location.hostname;
          return this.hostname(country, city, hostname);
        }

        if (location.city) {
          const [country, city] = location.city;
          return this.city(country, city);
        }

        if (location.country) {
          return this.country(location.country);
        }

        throw new Error(
          'Unsupported value of RelayLocation' + (location && JSON.stringify(location)),
        );
      },
    };
  }

  get tunnel(): TunnelBuilder<NormalRelaySettingsBuilder> {
    const updateOpenvpn = (next) => {
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
      openvpn: (configurator) => {
        const openvpnBuilder = {
          get port() {
            const apply = (port) => {
              updateOpenvpn({ port });
              return this;
            };
            return {
              exact: (value: number) => apply({ only: value }),
              any: () => apply('any'),
            };
          },
          get protocol() {
            const apply = (protocol) => {
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
