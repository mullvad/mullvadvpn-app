import {
  type CustomLists,
  IRelayList,
  IWireguardEndpointData,
  type Recents,
} from '../../src/shared/daemon-rpc-types';

const wireguardEndpointData: IWireguardEndpointData = {
  portRanges: [],
  udp2tcpPorts: [],
};

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
              daita: true,
              lwo: true,
            },
            {
              hostname: 'mullvad-wireguard-2',
              provider: 'mullvad',
              ipv4AddrIn: '10.0.0.2',
              includeInCountry: true,
              active: true,
              weight: 0,
              owned: true,
              daita: true,
              lwo: false,
            },
            {
              hostname: 'another-provider-wireguard-1',
              provider: 'another-provider',
              ipv4AddrIn: '10.0.0.3',
              includeInCountry: true,
              active: true,
              weight: 0,
              owned: false,
              daita: true,
              lwo: false,
            },
            {
              hostname: 'mullvad-wireguard-quic-1',
              provider: 'mullvad',
              ipv4AddrIn: '10.0.0.4',
              includeInCountry: true,
              active: true,
              weight: 0,
              owned: true,
              daita: true,
              quic: {
                addrIn: ['10.0.0.4'],
                domain: '',
                token: '',
              },
              lwo: false,
            },
          ],
        },
      ],
    },
    {
      name: 'Denmark',
      code: 'dk',
      cities: [],
    },
  ],
};

const customLists: CustomLists = [
  {
    id: 'custom-list-1',
    name: 'Custom List 1',
    locations: [],
  },
];

const recents: Recents = {
  entries: [
    {
      country: relayList.countries[0].code,
    },
  ],
  exits: [
    {
      country: relayList.countries[0].code,
    },
    {
      customList: customLists[0].id,
    },
    {
      country: relayList.countries[0].code,
      city: relayList.countries[0].cities[0].code,
    },
  ],
};

export const mockData = {
  relayList,
  wireguardEndpointData,
  customLists,
  recents,
};
