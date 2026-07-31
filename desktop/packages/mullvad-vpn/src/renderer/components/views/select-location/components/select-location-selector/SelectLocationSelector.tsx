import { messages } from '../../../../../../shared/gettext';
import { useActiveFilters } from '../../../../../features/locations/hooks';
import { LocationType } from '../../../../../features/locations/types';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { LocationSelector } from '../../../../../lib/components/location-selector';
import { useIsLocationSelectorExpanded } from '../../hooks';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { FilterChips } from '../filter-chips';
import { SelectLocationSelectorEntryItem, SelectLocationSelectorExitItem } from './components';
import {
  useHandleSelectedItemChange,
  useLocationSelectorVariant,
  useShowSelectLocationSelectorEntryItem,
  useShowSelectLocationSelectorExitItem,
} from './hooks';

export function SelectLocationSelector() {
  const { locationType } = useSelectLocationViewContext();
  const { isAnyFilterActive: showFilterChips } = useActiveFilters(locationType);
  const expanded = useIsLocationSelectorExpanded();
  const handleSelectedItemChange = useHandleSelectedItemChange();

  const selectedItem = locationType === LocationType.entry ? 'entry' : 'exit';

  const showSelectLocationSelectorEntryItem = useShowSelectLocationSelectorEntryItem();
  const showSelectLocationSelectorExitItem = useShowSelectLocationSelectorExitItem();
  const variant = useLocationSelectorVariant();

  return (
    <LocationSelector
      selectedItem={selectedItem}
      onSelectedItemChange={handleSelectedItemChange}
      expanded={expanded}
      variant={variant}>
      <LocationSelector.Row position="top">
        <LocationSelector.Row.Content>
          <LocationSelector.Row.Icon icon="device" />
          <LocationSelector.Row.Label>{messages.gettext('Your device')}</LocationSelector.Row.Label>
        </LocationSelector.Row.Content>
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
        <FlexColumn gap="small">
          <LocationSelector.Row.Content>
            <LocationSelector.Row.Icon icon="internet" />
            <LocationSelector.Row.Label>{messages.gettext('Internet')}</LocationSelector.Row.Label>
          </LocationSelector.Row.Content>
          {showFilterChips && <FilterChips />}
        </FlexColumn>
      </LocationSelector.Row>
    </LocationSelector>
  );
}
