import { AnimatePresence } from 'motion/react';
import { useActiveFilters } from '../../../../../features/locations/hooks';
import { LocationSelector } from '../../../../../lib/components/location-selector';
import { useEntryType, useLocationSelectorItems } from '../../hooks';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { FilterChips } from '../filter-chips';
import { SelectLocationSelectorDeviceRow } from '../select-location-selector-device-row';
import { SelectLocationSelectorInternetRow } from '../select-location-selector-internet-row';
import {
  useHandleSelectedItemChange,
  useIsExpanded,
  useLocationSelectorVariant,
  useSelectedItem,
} from './hooks';

export function SelectLocationSelector() {
  const { locationType } = useSelectLocationViewContext();
  const handleSelectedItemChange = useHandleSelectedItemChange();
  const isExpanded = useIsExpanded();
  const selectedItem = useSelectedItem();
  const variant = useLocationSelectorVariant();
  const items = useLocationSelectorItems();

  const { isAnyFilterActive } = useActiveFilters(locationType);
  const entryType = useEntryType();
  const showFilterChips = isAnyFilterActive && entryType !== 'entryAutomatic';

  return (
    <LocationSelector
      selectedItem={selectedItem}
      onSelectedItemChange={handleSelectedItemChange}
      expanded={isExpanded}
      variant={variant}>
      <SelectLocationSelectorDeviceRow />
      <LocationSelector.Items>{Object.values(items)}</LocationSelector.Items>
      <SelectLocationSelectorInternetRow />
      <AnimatePresence initial={false}>
        {showFilterChips && <FilterChips key="location-selector-filter-chips" />}
      </AnimatePresence>
    </LocationSelector>
  );
}
