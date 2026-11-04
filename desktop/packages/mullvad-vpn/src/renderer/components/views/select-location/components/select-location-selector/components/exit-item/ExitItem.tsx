import { messages } from '../../../../../../../../shared/gettext';
import { useExitLocation } from '../../../../../../../features/locations/hooks';
import { useGetIsLocationValid, useLocationName } from '../../hooks';
import { type SelectLocationSelectorItemProps, TextFieldItem } from '../text-field-item';

export type ExitItemProps = Omit<
  SelectLocationSelectorItemProps,
  'name' | 'placeholder' | 'value' | 'id' | 'type'
>;

export function ExitItem(props: ExitItemProps) {
  const { exitLocation } = useExitLocation();
  const defaultValue = useLocationName(exitLocation);
  const validate = useGetIsLocationValid('exit');

  return (
    <TextFieldItem
      id="exit"
      type="exit"
      aria-label={messages.gettext('Search exit location or server')}
      placeholder={messages.gettext('Search exit location or server')}
      defaultValue={defaultValue}
      validate={validate}
      {...props}
    />
  );
}
