import {
  RelayLocationCity as DaemonRelayLocationCity,
  RelayLocationCountry as DaemonRelayLocationCountry,
  RelayLocationCustomList as DaemonRelayLocationCustomList,
  RelayLocationRelay as DaemonRelayLocationRelay,
} from '../../../../shared/daemon-rpc-types';
import {
  IRelayLocationCityRedux,
  IRelayLocationCountryRedux,
  IRelayLocationRelayRedux,
} from '../../../redux/settings/reducers';
import type { SpecialLocationRowInnerProps } from './components';

export enum LocationType {
  entry = 0,
  exit,
}

export interface LocationVisibility {
  visible: boolean;
}

interface CommonLocationSpecification {
  label: string;
  selected: boolean;
  disabled?: boolean;
  disabledReason?: DisabledReason;
}

export interface SpecialLocation<T> extends CommonLocationSpecification {
  value: T;
  component: React.ComponentType<SpecialLocationRowInnerProps<T>>;
}

export interface RelayLocationCountryWithVisibility
  extends IRelayLocationCountryRedux,
    LocationVisibility {
  cities: Array<RelayLocationCityWithVisibility>;
}

export interface RelayLocationCityWithVisibility
  extends IRelayLocationCityRedux,
    LocationVisibility {
  relays: Array<RelayLocationRelayWithVisibility>;
}

export type RelayLocationRelayWithVisibility = IRelayLocationRelayRedux & LocationVisibility;

type LocationState = {
  visible: boolean;
  active: boolean;
  expanded: boolean;
  label: string;
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
