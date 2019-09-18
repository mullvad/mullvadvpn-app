import * as React from 'react';
import { RelayLocation } from '../../shared/daemon-rpc-types';
import { IRelayLocationRedux } from '../redux/settings/reducers';
import LocationList, {
  LocationSelection,
  LocationSelectionType,
  RelayLocations,
} from './LocationList';

interface IExitLocationsProps {
  source: IRelayLocationRedux[];
  defaultExpandedLocations?: RelayLocation[];
  selectedValue?: RelayLocation;
  selectedElementRef?: React.Ref<React.ReactInstance>;
  onSelect?: (value: LocationSelection<never>) => void;
}

const ExitLocations = React.forwardRef(function ExitLocationsT(
  props: IExitLocationsProps,
  ref: React.Ref<LocationList<never>>,
) {
  const selectedValue: LocationSelection<never> | undefined = props.selectedValue
    ? { type: LocationSelectionType.relay, value: props.selectedValue }
    : undefined;

  return (
    <LocationList
      ref={ref}
      defaultExpandedLocations={props.defaultExpandedLocations}
      selectedValue={selectedValue}
      selectedElementRef={props.selectedElementRef}
      onSelect={props.onSelect}>
      <RelayLocations source={props.source} />
    </LocationList>
  );
});

export default ExitLocations;
