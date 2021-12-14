import * as React from 'react';
import { LiftedConstraint, RelayLocation } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { IRelayLocationRedux } from '../redux/settings/reducers';
import LocationList, {
  LocationSelection,
  LocationSelectionType,
  RelayLocations,
  SpecialLocation,
  SpecialLocationIcon,
  SpecialLocations,
} from './LocationList';

export enum SpecialBridgeLocationType {
  closestToExit = 0,
}

interface IBridgeLocationsProps {
  source: IRelayLocationRedux[];
  locale: string;
  defaultExpandedLocations?: RelayLocation[];
  selectedValue?: LiftedConstraint<RelayLocation>;
  selectedElementRef?: React.Ref<React.ReactInstance>;
  onSelect?: (value: LocationSelection<SpecialBridgeLocationType>) => void;
  onWillExpand?: (locationRect: DOMRect, expandedContentHeight: number) => void;
  onTransitionEnd?: () => void;
}

const BridgeLocations = React.forwardRef(function BridgeLocationsT(
  props: IBridgeLocationsProps,
  ref: React.Ref<LocationList<SpecialBridgeLocationType>>,
) {
  const selectedValue:
    | LocationSelection<SpecialBridgeLocationType>
    | undefined = props.selectedValue
    ? props.selectedValue === 'any'
      ? { type: LocationSelectionType.special, value: SpecialBridgeLocationType.closestToExit }
      : { type: LocationSelectionType.relay, value: props.selectedValue }
    : undefined;

  return (
    <LocationList
      ref={ref}
      defaultExpandedLocations={props.defaultExpandedLocations}
      selectedValue={selectedValue}
      selectedElementRef={props.selectedElementRef}
      onSelect={props.onSelect}>
      <SpecialLocations>
        <SpecialLocation
          icon={SpecialLocationIcon.geoLocation}
          value={SpecialBridgeLocationType.closestToExit}>
          {messages.pgettext('select-location-view', 'Closest to exit server')}
        </SpecialLocation>
      </SpecialLocations>
      <RelayLocations
        source={props.source}
        locale={props.locale}
        onWillExpand={props.onWillExpand}
        onTransitionEnd={props.onTransitionEnd}
      />
    </LocationList>
  );
});

export default BridgeLocations;
