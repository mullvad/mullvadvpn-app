import {
  ICustomList,
  RelayLocation,
  RelayLocationCity,
  RelayLocationCountry,
  RelayLocationCustomList,
  RelayLocationRelay,
} from '../../../shared/daemon-rpc-types';
import {
  IRelayLocationCityRedux,
  IRelayLocationCountryRedux,
  IRelayLocationRelayRedux,
} from '../../redux/settings/reducers';
import { SpecialLocationRowInnerProps } from './SpecialLocationList';

export enum LocationType {
  entry = 0,
  exit,
}

export type RelayList = GeographicalRelayList | Array<CustomListSpecification>;
export type GeographicalRelayList = Array<CountrySpecification>;

export enum SpecialBridgeLocationType {
  closestToExit,
  custom,
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

type GeographicalLocationSpecification =
  | CountrySpecification
  | CitySpecification
  | RelaySpecification;

export type LocationSpecification = GeographicalLocationSpecification | CustomListSpecification;

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

interface CommonNormalLocationSpecification
  extends CommonLocationSpecification,
    LocationVisibility {
  location: RelayLocation;
  disabled: boolean;
  active: boolean;
}

export interface CustomListSpecification extends CommonNormalLocationSpecification {
  location: RelayLocationCustomList;
  list: ICustomList;
  expanded: boolean;
  locations: Array<GeographicalLocationSpecification>;
}

export interface CountrySpecification
  extends Pick<IRelayLocationCountryRedux, 'name' | 'code'>,
    CommonNormalLocationSpecification {
  location: RelayLocationCountry;
  expanded: boolean;
  cities: Array<CitySpecification>;
}

export interface CitySpecification
  extends Pick<IRelayLocationCityRedux, 'name' | 'code'>,
    CommonNormalLocationSpecification {
  location: RelayLocationCity;
  expanded: boolean;
  relays: Array<RelaySpecification>;
}

export interface RelaySpecification
  extends Omit<IRelayLocationRelayRedux, 'ipv4AddrIn' | 'includeInCountry' | 'weight'>,
    CommonNormalLocationSpecification {
  location: RelayLocationRelay;
}

export enum DisabledReason {
  entry,
  exit,
  inactive,
}

export function getLocationChildren(location: LocationSpecification): Array<LocationSpecification> {
  if ('locations' in location) {
    return location.locations;
  } else if ('cities' in location) {
    return location.cities;
  } else if ('relays' in location) {
    return location.relays;
  } else {
    return [];
  }
}
