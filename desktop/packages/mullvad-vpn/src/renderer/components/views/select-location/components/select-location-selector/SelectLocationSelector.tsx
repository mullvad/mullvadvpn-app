import { messages } from '../../../../../../shared/gettext';
import { LocationType } from '../../../../../features/locations/types';
import { useMultihop } from '../../../../../features/multihop/hooks';
import { LocationSelector } from '../../../../../lib/components/location-selector';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { SelectLocationSelectorEntryItem, SelectLocationSelectorExitItem } from './components';
import { useHandleSelectedItemChange, useIsLocationSelectorExpanded } from './hooks';
import {
  SelectLocationSelectorProvider,
  useSelectLocationSelectorContext,
} from './SelectLocationSelectorContext';

function SelectLocationSelectorImpl() {
  const { multihop } = useMultihop();
  const { locationType } = useSelectLocationViewContext();
  const { isolatedItem } = useSelectLocationSelectorContext();

  const expanded = useIsLocationSelectorExpanded();
  const handleSelectedItemChange = useHandleSelectedItemChange();

  const selectedItem = locationType === LocationType.entry ? 'entry' : 'exit';

  const showSelectLocationSelectorEntryItem = multihop !== 'never' && isolatedItem !== 'exit';
  const showSelectLocationSelectorExitItem = isolatedItem !== 'entry';

  return (
    <LocationSelector
      selectedItem={selectedItem}
      onSelectedItemChange={handleSelectedItemChange}
      expanded={expanded}
      variant={multihop === 'never' ? 'primary' : 'secondary'}>
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

export function SelectLocationSelector() {
  return (
    <SelectLocationSelectorProvider>
      <SelectLocationSelectorImpl />
    </SelectLocationSelectorProvider>
  );
}
