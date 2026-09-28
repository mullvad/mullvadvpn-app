import { messages } from '../../../../../../../../shared/gettext';
import { useSelectedLocations } from '../../../../../../../features/locations/hooks';
import { useLocationName } from '../../hooks';
import { type SelectLocationSelectorItemProps, TextFieldItem } from '../text-field-item';
import { useIsValidExitLocation } from './hooks';

export type ExitItemProps = Omit<
  SelectLocationSelectorItemProps,
  'name' | 'placeholder' | 'value' | 'id' | 'type'
>;

export function ExitItem(props: ExitItemProps) {
  const { exit } = useSelectedLocations();
  const defaultValue = useLocationName(exit);

  const isValidExitLocation = useIsValidExitLocation();

  return (
    <TextFieldItem
      id="exit"
      type="exit"
      aria-label={messages.gettext('Search exit location or server, press enter to search')}
      placeholder={messages.gettext('Search exit location or server')}
      defaultValue={defaultValue}
      invalid={!isValidExitLocation}
      {...props}
    />
  );
}
