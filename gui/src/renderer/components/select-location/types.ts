import { RelayLocation } from '../../../shared/daemon-rpc-types';
import {
  IRelayLocationCityRedux,
  IRelayLocationRedux,
  IRelayLocationRelayRedux,
} from '../../redux/settings/reducers';

export interface Relay extends IRelayLocationRelayRedux {
  label: string;
  location: RelayLocation;
  disabled: boolean;
}

export interface City extends Omit<IRelayLocationCityRedux, 'relays'> {
  label: string;
  location: RelayLocation;
  active: boolean;
  disabled: boolean;
  expanded: boolean;
  relays: Array<Relay>;
}

export interface Country extends Omit<IRelayLocationRedux, 'cities'> {
  label: string;
  location: RelayLocation;
  active: boolean;
  disabled: boolean;
  expanded: boolean;
  cities: Array<City>;
}
