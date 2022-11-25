import { RelayLocation } from '../../../shared/daemon-rpc-types';
import {
  IRelayLocationCityRedux,
  IRelayLocationRedux,
  IRelayLocationRelayRedux,
} from '../../redux/settings/reducers';

export enum LocationType {
  entry = 0,
  exit,
}

export enum LocationSelectionType {
  relay = 'relay',
  special = 'special',
}

export type LocationSelection<T> =
  | { type: LocationSelectionType.special; value: T }
  | { type: LocationSelectionType.relay; value: RelayLocation };

export type LocationList<T> = Array<CountrySpecification | SpecialLocation<T>>;
export type RelayList = Array<CountrySpecification>;

export enum SpecialBridgeLocationType {
  closestToExit = 0,
}

export enum SpecialLocationIcon {
  geoLocation = 'icon-nearest',
}

export interface SpecialLocation<T> {
  type: LocationSelectionType.special;
  label: string;
  icon?: SpecialLocationIcon;
  info?: string;
  value: T;
  disabled?: boolean;
  selected: boolean;
}

export type LocationSpecification = CountrySpecification | CitySpecification | RelaySpecification;

export interface CountrySpecification extends Omit<IRelayLocationRedux, 'cities'> {
  type: LocationSelectionType.relay;
  label: string;
  location: RelayLocation;
  active: boolean;
  disabled: boolean;
  expanded: boolean;
  selected: boolean;
  cities: Array<CitySpecification>;
}

export interface CitySpecification extends Omit<IRelayLocationCityRedux, 'relays'> {
  label: string;
  location: RelayLocation;
  active: boolean;
  disabled: boolean;
  expanded: boolean;
  selected: boolean;
  relays: Array<RelaySpecification>;
}

export interface RelaySpecification extends IRelayLocationRelayRedux {
  label: string;
  location: RelayLocation;
  disabled: boolean;
  selected: boolean;
}

export enum DisabledReason {
  entry,
  exit,
  inactive,
}

export function getLocationChildren(location: LocationSpecification): Array<LocationSpecification> {
  if ('cities' in location) {
    return location.cities;
  } else if ('relays' in location) {
    return location.relays;
  } else {
    return [];
  }
}
