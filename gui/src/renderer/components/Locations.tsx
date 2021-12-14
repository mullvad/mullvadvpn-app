import * as React from 'react';
import { RelayLocation } from '../../shared/daemon-rpc-types';
import { IRelayLocationRedux } from '../redux/settings/reducers';
import LocationList, {
  DisabledReason,
  LocationSelection,
  LocationSelectionType,
  RelayLocations,
} from './LocationList';

interface ILocationsProps {
  source: IRelayLocationRedux[];
  locale: string;
  defaultExpandedLocations?: RelayLocation[];
  selectedValue?: RelayLocation;
  disabledLocation?: { location: RelayLocation; reason: DisabledReason };
  selectedElementRef?: React.Ref<React.ReactInstance>;
  onSelect?: (value: LocationSelection<never>) => void;
  onWillExpand?: (locationRect: DOMRect, expandedContentHeight: number) => void;
  onTransitionEnd?: () => void;
}

function Locations(props: ILocationsProps, ref: React.Ref<LocationList<never>>) {
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
      <RelayLocations
        source={props.source}
        locale={props.locale}
        disabledLocation={props.disabledLocation}
        onWillExpand={props.onWillExpand}
        onTransitionEnd={props.onTransitionEnd}
      />
    </LocationList>
  );
}

export const ExitLocations = React.forwardRef(Locations);
export const EntryLocations = React.forwardRef(Locations);
