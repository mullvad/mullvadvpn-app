import { AnimatePresence } from 'motion/react';

import {
  LocationSelector,
  type LocationSelectorProps,
} from '../../../../../lib/components/location-selector';
import { useLocationSelectorItems, useShowFilterChips } from '../../hooks';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { FilterChips } from '../filter-chips';
import { SelectLocationSelectorDeviceRow } from '../select-location-selector-device-row';
import { SelectLocationSelectorInternetRow } from '../select-location-selector-internet-row';
import { useIsExpanded, useLocationSelectorVariant } from './hooks';

export type SelectLocationSelectorProps = {
  expanded?: LocationSelectorProps['expanded'];
  showFilterChips?: boolean;
};

export function SelectLocationSelector({
  expanded: expandedProp,
  showFilterChips: showFilterChipsProp,
}: SelectLocationSelectorProps) {
  const { locationType, setLocationType } = useSelectLocationViewContext();
  const variant = useLocationSelectorVariant();
  const items = useLocationSelectorItems();

  const showFilterChips = useShowFilterChips();
  const expanded = useIsExpanded();

  return (
    <LocationSelector
      selectedItem={locationType}
      onSelectedItemChange={setLocationType}
      expanded={expandedProp ?? expanded}
      variant={variant}>
      <SelectLocationSelectorDeviceRow />
      <LocationSelector.Items>{Object.values(items)}</LocationSelector.Items>
      <SelectLocationSelectorInternetRow />
      <AnimatePresence initial={false}>
        {(showFilterChipsProp ?? showFilterChips) && (
          <FilterChips key="location-selector-filter-chips" />
        )}
      </AnimatePresence>
    </LocationSelector>
  );
}
