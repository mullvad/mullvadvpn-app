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

export enum LocationType {
  entry = 0,
  exit,
}

export type RelayList = GeographicalRelayList | Array<CustomListSpecification>;
export type GeographicalRelayList = Array<CountrySpecification>;

export enum SpecialBridgeLocationType {
  closestToExit = 0,
}

export enum SpecialLocationIcon {
  geoLocation = 'icon-nearest',
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
  icon?: SpecialLocationIcon;
  info?: string;
  value: T;
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

export interface CustomListSpecification
  extends Omit<ICustomList, 'locations'>,
    CommonNormalLocationSpecification {
  location: RelayLocationCustomList;
  list: ICustomList;
  expanded: boolean;
  locations: Array<GeographicalLocationSpecification>;
}

export interface CountrySpecification
  extends Omit<IRelayLocationCountryRedux, 'cities'>,
    CommonNormalLocationSpecification {
  location: RelayLocationCountry;
  expanded: boolean;
  cities: Array<CitySpecification>;
}

export interface CitySpecification
  extends Omit<IRelayLocationCityRedux, 'relays'>,
    CommonNormalLocationSpecification {
  location: RelayLocationCity;
  expanded: boolean;
  relays: Array<RelaySpecification>;
}

export interface RelaySpecification
  extends IRelayLocationRelayRedux,
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
