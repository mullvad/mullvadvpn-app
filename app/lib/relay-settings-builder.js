// @flow

import type {
  RelayLocation,
  RelayProtocol,
  RelaySettingsUpdate,
  RelaySettingsNormalUpdate,
  RelaySettingsCustom,
} from './daemon-rpc';

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
      any: () => {
        this._payload.location = 'any';
        return this;
      },
      fromRaw: function(location: 'any' | RelayLocation) {
        if (location === 'any') {
          return this.any();
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

type CustomOpenVPNConfigurator<Self> = {
  port: (port: number) => Self,
  protocol: (protocol: RelayProtocol) => Self,
};

type CustomTunnelBuilder<Self> = {
  openvpn: (configurator: (CustomOpenVPNConfigurator<*>) => void) => Self,
};

class CustomRelaySettingsBuilder {
  _payload: RelaySettingsCustom = {
    host: '',
    tunnel: {
      openvpn: {
        port: 0,
        protocol: 'udp',
      },
    },
  };

  build(): RelaySettingsUpdate {
    return {
      custom_tunnel_endpoint: this._payload,
    };
  }

  host(value: string) {
    this._payload.host = value;
    return this;
  }

  get tunnel(): CustomTunnelBuilder<CustomRelaySettingsBuilder> {
    const updateOpenvpn = (next) => {
      const tunnel = this._payload.tunnel || {};
      const prev = tunnel.openvpn || {};
      this._payload.tunnel = {
        openvpn: { ...prev, ...next },
      };
    };

    return {
      openvpn: (configurator) => {
        configurator({
          port: function(port: number) {
            updateOpenvpn({ port });
            return this;
          },
          protocol: function(protocol: RelayProtocol) {
            updateOpenvpn({ protocol });
            return this;
          },
        });
        return this;
      },
    };
  }
}

export default {
  normal: () => new NormalRelaySettingsBuilder(),
  custom: () => new CustomRelaySettingsBuilder(),
};
