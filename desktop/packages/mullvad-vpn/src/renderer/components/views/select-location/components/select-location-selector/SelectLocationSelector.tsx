import { LocationType } from '../../../../../features/locations/types';
import { LocationSelector } from '../../../../../lib/components/location-selector';
import { useIsLocationSelectorExpanded } from '../../hooks';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { SelectLocationSelectorDeviceRow } from '../select-location-selector-device-row';
import { SelectLocationSelectorInternetRow } from '../select-location-selector-internet-row';
import { SelectLocationSelectorEntryItem, SelectLocationSelectorExitItem } from './components';
import {
  useHandleSelectedItemChange,
  useLocationSelectorVariant,
  useShowSelectLocationSelectorEntryItem,
  useShowSelectLocationSelectorExitItem,
} from './hooks';

export function SelectLocationSelector() {
  const { locationType } = useSelectLocationViewContext();
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
      <SelectLocationSelectorDeviceRow />
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
      <SelectLocationSelectorInternetRow />
    </LocationSelector>
  );
}
