import {
  RelayLocationCity as DaemonRelayLocationCity,
  RelayLocationCountry as DaemonRelayLocationCountry,
  RelayLocationCustomList as DaemonRelayLocationCustomList,
  RelayLocationRelay as DaemonRelayLocationRelay,
} from '../../../../shared/daemon-rpc-types';

export enum LocationType {
  entry = 0,
  exit,
}

type LocationState = {
  active: boolean;
  expanded: boolean;
  label: string;
  searchText: string;
  selected: boolean;
  disabled?: boolean;
  disabledReason?: DisabledReason;
};

export type CustomListLocation = LocationState & {
  type: 'customList';
  details: DaemonRelayLocationCustomList;
  locations: Array<GeographicalLocation>;
};

export type CountryLocation = LocationState & {
  type: 'country';
  details: DaemonRelayLocationCountry;
  cities: CityLocation[];
};

export type CityLocation = LocationState & {
  type: 'city';
  details: DaemonRelayLocationCity;
  relays: RelayLocation[];
};

export type RelayLocation = LocationState & {
  type: 'relay';
  details: DaemonRelayLocationRelay;
};

export type AnyLocation = CustomListLocation | CountryLocation | CityLocation | RelayLocation;
export type GeographicalLocation = CountryLocation | CityLocation | RelayLocation;

export enum DisabledReason {
  entry,
  exit,
  inactive,
}

export function getLocationChildrenByType(location: AnyLocation): GeographicalLocation[] {
  if (location.type === 'customList') {
    return location.locations;
  } else if (location.type === 'country') {
    return location.cities;
  } else if (location.type === 'city') {
    return location.relays;
  } else {
    return [];
  }
}
