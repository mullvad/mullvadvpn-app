import { messages } from '../../../../../../../../shared/gettext';
import { useSelectedLocations } from '../../../../../../../features/locations/hooks';
import { useLocationName } from '../../hooks';
import {
  SelectLocationSelectorItem,
  type SelectLocationSelectorItemProps,
} from '../select-location-selector-item';

export type SelectLocationSelectorEntryItemProps = Omit<
  SelectLocationSelectorItemProps,
  'name' | 'placeholder' | 'value' | 'id'
>;

export function SelectLocationSelectorEntryItem(props: SelectLocationSelectorEntryItemProps) {
  const { entry } = useSelectedLocations();
  const defaultValue = useLocationName(entry);

  return (
    <SelectLocationSelectorItem
      id="entry"
      aria-label={messages.gettext('Search entry location, press enter to edit')}
      placeholder={messages.gettext('Search entry location or server')}
      defaultValue={defaultValue}
      {...props}
    />
  );
}
