import {
  LocationSelector,
  type LocationSelectorProps,
} from '../../../../../lib/components/location-selector';
import { useLocationSelectorItems, useShowFilterChips } from '../../hooks';
import { FilterChips } from '../filter-chips';
import { SelectLocationSelectorDeviceRow } from '../select-location-selector-device-row';
import { SelectLocationSelectorInternetRow } from '../select-location-selector-internet-row';
import {
  useHandleSelectedItemChange,
  useIsExpanded,
  useLocationSelectorVariant,
  useSelectedItem,
} from './hooks';

export type SelectLocationSelectorProps = {
  expanded?: LocationSelectorProps['expanded'];
  showFilterChips?: boolean;
};

export function SelectLocationSelector({
  expanded: expandedProp,
  showFilterChips: showFilterChipsProp,
}: SelectLocationSelectorProps) {
  const handleSelectedItemChange = useHandleSelectedItemChange();
  const selectedItem = useSelectedItem();
  const variant = useLocationSelectorVariant();
  const items = useLocationSelectorItems();

  const showFilterChips = useShowFilterChips();
  const expanded = useIsExpanded();

  return (
    <LocationSelector
      selectedItem={selectedItem}
      onSelectedItemChange={handleSelectedItemChange}
      expanded={expandedProp ?? expanded}
      variant={variant}>
      <SelectLocationSelectorDeviceRow />
      <LocationSelector.Items>{Object.values(items)}</LocationSelector.Items>
      <SelectLocationSelectorInternetRow />
      {(showFilterChipsProp ?? showFilterChips) && (
        <FilterChips key="location-selector-filter-chips" />
      )}
    </LocationSelector>
  );
}
