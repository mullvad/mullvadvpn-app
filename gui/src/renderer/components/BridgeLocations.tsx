import * as React from 'react';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { LiftedConstraint, RelayLocation } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { useBoolean } from '../lib/utilityHooks';
import { IRelayLocationRedux } from '../redux/settings/reducers';
import * as AppButton from './AppButton';
import ImageView from './ImageView';
import LocationList, {
  LocationSelection,
  LocationSelectionType,
  RelayLocations,
  SpecialLocation,
  SpecialLocationIcon,
  SpecialLocations,
} from './LocationList';
import { ModalAlert, ModalAlertType } from './Modal';

export enum SpecialBridgeLocationType {
  closestToExit = 0,
}

const StyledInfoIcon = styled(ImageView)({
  marginRight: '9px',
});

const StyledAutomaticLabel = styled.div({
  display: 'flex',
  justifyContent: 'space-between',
});

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
  const [automaticInfoVisible, showAutomaticInfo, hideAutomaticInfo] = useBoolean(false);

  const selectedValue:
    | LocationSelection<SpecialBridgeLocationType>
    | undefined = props.selectedValue
    ? props.selectedValue === 'any'
      ? { type: LocationSelectionType.special, value: SpecialBridgeLocationType.closestToExit }
      : { type: LocationSelectionType.relay, value: props.selectedValue }
    : undefined;

  return (
    <>
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
            <StyledAutomaticLabel>
              {messages.gettext('Automatic')}
              <StyledInfoIcon
                source="icon-info"
                width={18}
                tintColor={colors.white}
                tintHoverColor={colors.white80}
                onClick={showAutomaticInfo}
              />
            </StyledAutomaticLabel>
          </SpecialLocation>
        </SpecialLocations>
        <RelayLocations
          source={props.source}
          locale={props.locale}
          onWillExpand={props.onWillExpand}
          onTransitionEnd={props.onTransitionEnd}
        />
      </LocationList>

      <ModalAlert
        isOpen={automaticInfoVisible}
        message={messages.pgettext(
          'select-location-view',
          'The app selects a random bridge server, but servers have a higher probability the closer they are to you.',
        )}
        type={ModalAlertType.info}
        buttons={[
          <AppButton.BlueButton key="back" onClick={hideAutomaticInfo}>
            {messages.gettext('Got it!')}
          </AppButton.BlueButton>,
        ]}
        close={hideAutomaticInfo}
      />
    </>
  );
});

export default BridgeLocations;
