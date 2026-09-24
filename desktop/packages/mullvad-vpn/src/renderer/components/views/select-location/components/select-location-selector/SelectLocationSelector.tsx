import { LocationSelector } from '../../../../../lib/components/location-selector';
import { SelectLocationSelectorDeviceRow } from '../select-location-selector-device-row';
import { SelectLocationSelectorInternetRow } from '../select-location-selector-internet-row';
import { AutomaticEntryItem, EntryItem, ExitItem } from './components';
import {
  useHandleSelectedItemChange,
  useIsExpanded,
  useLocationSelectorVariant,
  useSelectedItem,
  useShowAutomaticEntryItem,
  useShowEntryItem,
  useShowExitItem,
} from './hooks';

export function SelectLocationSelector() {
  const handleSelectedItemChange = useHandleSelectedItemChange();
  const isExpanded = useIsExpanded();
  const selectedItem = useSelectedItem();
  const showAutomaticEntryItem = useShowAutomaticEntryItem();
  const showEntryItem = useShowEntryItem();
  const showExitItem = useShowExitItem();
  const variant = useLocationSelectorVariant(showAutomaticEntryItem);

  return (
    <LocationSelector
      selectedItem={selectedItem}
      onSelectedItemChange={handleSelectedItemChange}
      expanded={isExpanded}
      variant={variant}>
      <SelectLocationSelectorDeviceRow />
      <LocationSelector.Items>
        {/* Assign keys to each item to ensure proper rendering with motion components */}
        {showAutomaticEntryItem ? <AutomaticEntryItem key="entryAutomatic" /> : null}
        {showEntryItem ? <EntryItem key="entry" /> : null}
        {showExitItem ? <ExitItem key="exit" /> : null}
      </LocationSelector.Items>
      <SelectLocationSelectorInternetRow />
    </LocationSelector>
  );
}
