import { messages } from '../../../../../../../../shared/gettext';
import { useSelectedLocations } from '../../../../../../../features/locations/hooks';
import { useLocationName } from '../../hooks';
import {
  SelectLocationSelectorItem,
  type SelectLocationSelectorItemProps,
} from '../select-location-selector-item';

export type SelectLocationSelectorExitItemProps = Omit<
  SelectLocationSelectorItemProps,
  'name' | 'placeholder' | 'value' | 'id'
>;

export function SelectLocationSelectorExitItem(props: SelectLocationSelectorExitItemProps) {
  const { exit } = useSelectedLocations();
  const defaultValue = useLocationName(exit);

  return (
    <SelectLocationSelectorItem
      id="exit"
      aria-label={messages.gettext('Search exit location or server, press enter to search')}
      placeholder={messages.gettext('Search exit location or server')}
      defaultValue={defaultValue}
      {...props}
    />
  );
}
