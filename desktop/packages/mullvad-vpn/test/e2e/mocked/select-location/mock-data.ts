import { IRelayList } from '../../../../src/shared/daemon-rpc-types';

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
              hostname: 'my-test-relay1',
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
              hostname: 'my-test-relay2',
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
              hostname: 'se-got-wg-103',
              provider: 'another-provider',
              ipv4AddrIn: '10.0.0.3',
              includeInCountry: true,
              active: true,
              weight: 0,
              owned: false,
              endpointType: 'wireguard',
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
