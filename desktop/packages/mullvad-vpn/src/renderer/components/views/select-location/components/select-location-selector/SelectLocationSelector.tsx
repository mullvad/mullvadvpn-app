import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { LocationType } from '../../../../../features/locations/types';
import { LocationSelector } from '../../../../../lib/components/location-selector';
import { colors, spacings } from '../../../../../lib/foundations';
import { useIsLocationSelectorExpanded } from '../../hooks';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { SelectLocationSelectorEntryItem, SelectLocationSelectorExitItem } from './components';
import {
  useHandleSelectedItemChange,
  useLocationSelectorVariant,
  useShowSelectLocationSelectorEntryItem,
  useShowSelectLocationSelectorExitItem,
} from './hooks';

const StyledLocationSelectorBackgroundContainer = styled.div`
  background: ${colors.darkBlue};
  padding-bottom: ${spacings.medium};
`;

export function SelectLocationSelector() {
  const { locationType } = useSelectLocationViewContext();
  const expanded = useIsLocationSelectorExpanded();
  const handleSelectedItemChange = useHandleSelectedItemChange();

  const selectedItem = locationType === LocationType.entry ? 'entry' : 'exit';

  const showSelectLocationSelectorEntryItem = useShowSelectLocationSelectorEntryItem();
  const showSelectLocationSelectorExitItem = useShowSelectLocationSelectorExitItem();
  const variant = useLocationSelectorVariant();

  return (
    <StyledLocationSelectorBackgroundContainer>
      <LocationSelector
        selectedItem={selectedItem}
        onSelectedItemChange={handleSelectedItemChange}
        expanded={expanded}
        variant={variant}>
        <LocationSelector.Row position="top">
          <LocationSelector.Row.Icon icon="device" />
          <LocationSelector.Row.Label>{messages.gettext('Your device')}</LocationSelector.Row.Label>
        </LocationSelector.Row>
        <LocationSelector.Items>
          {/* NOTE: The components must have a `key` assigned as the `LocationSelector.Items`
           * component uses `motion` components under the hood, which requires all children
           * to use keys.
           */}
          {showSelectLocationSelectorEntryItem ? (
            <SelectLocationSelectorEntryItem key="entry" type="entry" />
          ) : null}
          {showSelectLocationSelectorExitItem ? (
            <SelectLocationSelectorExitItem key="exit" type="exit" />
          ) : null}
        </LocationSelector.Items>
        <LocationSelector.Row position="bottom">
          <LocationSelector.Row.Icon icon="internet" />
          <LocationSelector.Row.Label>{messages.gettext('Internet')}</LocationSelector.Row.Label>
        </LocationSelector.Row>
      </LocationSelector>
    </StyledLocationSelectorBackgroundContainer>
  );
}
