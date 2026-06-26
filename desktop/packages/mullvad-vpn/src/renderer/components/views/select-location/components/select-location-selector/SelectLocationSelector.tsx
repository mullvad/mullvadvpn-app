import { messages } from '../../../../../../shared/gettext';
import { LocationType } from '../../../../../features/locations/types';
import { LocationSelector } from '../../../../../lib/components/location-selector';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { SelectLocationSelectorEntryItem, SelectLocationSelectorExitItem } from './components';
import {
  useHandleSelectedItemChange,
  useIsLocationSelectorExpanded,
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
  );
}
