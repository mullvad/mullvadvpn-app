import { IRelayList } from '../../src/shared/daemon-rpc-types';

const relayList: IRelayList = {
  countries: [
    {
      name: 'Sweden',
      code: 'se',
      cities: [
        {
          name: 'Gothenburg',
          code: 'got',
          latitude: 58,
          longitude: 12,
          relays: [
            {
              hostname: 'mullvad-wireguard-1',
              provider: 'mullvad',
              ipv4AddrIn: '10.0.0.1',
              includeInCountry: true,
              active: true,
              weight: 0,
              owned: true,
              endpointType: 'wireguard',
              daita: true,
            },
            {
              hostname: 'mullvad-wireguard-23',
              provider: 'mullvad',
              ipv4AddrIn: '10.0.0.2',
              includeInCountry: true,
              active: true,
              weight: 0,
              owned: true,
              endpointType: 'wireguard',
              daita: true,
            },
            {
              hostname: 'another-provider-wireguard-1',
              provider: 'another-provider',
              ipv4AddrIn: '10.0.0.3',
              includeInCountry: true,
              active: true,
              weight: 0,
              owned: false,
              endpointType: 'wireguard',
              daita: true,
            },
            {
              hostname: 'mullvad-wireguard-quic-1',
              provider: 'mullvad',
              ipv4AddrIn: '10.0.0.4',
              includeInCountry: true,
              active: true,
              weight: 0,
              owned: true,
              endpointType: 'wireguard',
              daita: true,
              quic: {
                addrIn: [],
                domain: '',
                token: '',
              },
            },
            {
              hostname: 'mullvad-openvpn-1',
              provider: 'mullvad',
              ipv4AddrIn: '10.0.0.2',
              includeInCountry: true,
              active: true,
              weight: 0,
              owned: true,
              endpointType: 'openvpn',
              daita: true,
            },
          ],
        },
      ],
    },
  ],
};

export const mockData = {
  relayList,
};
