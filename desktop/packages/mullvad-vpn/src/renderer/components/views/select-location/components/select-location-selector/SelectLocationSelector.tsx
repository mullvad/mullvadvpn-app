import { LocationSelector } from '../../../../../lib/components/location-selector';
import { useLocationSelectorItems } from '../../hooks';
import { SelectLocationSelectorDeviceRow } from '../select-location-selector-device-row';
import { SelectLocationSelectorInternetRow } from '../select-location-selector-internet-row';
import {
  useHandleSelectedItemChange,
  useIsExpanded,
  useLocationSelectorVariant,
  useSelectedItem,
} from './hooks';

export function SelectLocationSelector() {
  const handleSelectedItemChange = useHandleSelectedItemChange();
  const isExpanded = useIsExpanded();
  const selectedItem = useSelectedItem();
  const variant = useLocationSelectorVariant();
  const items = useLocationSelectorItems();

  return (
    <LocationSelector
      selectedItem={selectedItem}
      onSelectedItemChange={handleSelectedItemChange}
      expanded={isExpanded}
      variant={variant}>
      <SelectLocationSelectorDeviceRow />
      <LocationSelector.Items>{Object.values(items)}</LocationSelector.Items>
      <SelectLocationSelectorInternetRow />
    </LocationSelector>
  );
}
