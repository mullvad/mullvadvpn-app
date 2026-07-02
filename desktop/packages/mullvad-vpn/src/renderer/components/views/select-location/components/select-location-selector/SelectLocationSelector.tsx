import { messages } from '../../../../../../shared/gettext';
import { LocationType } from '../../../../../features/locations/types';
import { useMultihop } from '../../../../../features/multihop/hooks';
import { LocationSelector } from '../../../../../lib/components/location-selector';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import {
  useHandleSelectedItemChange,
  useIsLocationSelectorExpanded,
  useLocationSelectorItems,
} from './hooks';
import { SelectLocationSelectorProvider } from './SelectLocationSelectorContext';

function SelectLocationSelectorImpl() {
  const { multihop } = useMultihop();
  const { locationType } = useSelectLocationViewContext();

  const items = useLocationSelectorItems();
  const expanded = useIsLocationSelectorExpanded();
  const handleSelectedItemChange = useHandleSelectedItemChange();

  const selectedItem = locationType === LocationType.entry ? 'entry' : 'exit';

  return (
    <LocationSelector
      selectedItem={selectedItem}
      onSelectedItemChange={handleSelectedItemChange}
      expanded={expanded}
      variant={multihop ? 'secondary' : 'primary'}>
      <LocationSelector.Row position="top">
        <LocationSelector.Row.Icon icon="device" />
        <LocationSelector.Row.Label>{messages.gettext('Your device')}</LocationSelector.Row.Label>
      </LocationSelector.Row>
      <LocationSelector.Items>{items}</LocationSelector.Items>
      <LocationSelector.Row position="bottom">
        <LocationSelector.Row.Icon icon="device" />
        <LocationSelector.Row.Label>{messages.gettext('Internet')}</LocationSelector.Row.Label>
      </LocationSelector.Row>
    </LocationSelector>
  );
}

export function SelectLocationSelector() {
  return (
    <SelectLocationSelectorProvider>
      <SelectLocationSelectorImpl />
    </SelectLocationSelectorProvider>
  );
}
