import { messages } from '../../../../../../../../shared/gettext';
import { useSelectedLocations } from '../../../../../../../features/locations/hooks';
import { LocationType } from '../../../../../../../features/locations/types';
import { useGetIsLocationValid, useLocationName } from '../../hooks';
import { type SelectLocationSelectorItemProps, TextFieldItem } from '../text-field-item';

export type EntryItemProps = Omit<
  SelectLocationSelectorItemProps,
  'name' | 'placeholder' | 'value' | 'id' | 'type'
>;

export function EntryItem(props: EntryItemProps) {
  const { entry } = useSelectedLocations();
  const defaultValue = useLocationName(entry);
  const validate = useGetIsLocationValid(LocationType.entry);

  return (
    <TextFieldItem
      id="entry"
      type="entry"
      aria-label={messages.gettext('Search entry location or server')}
      placeholder={messages.gettext('Search entry location or server')}
      defaultValue={defaultValue}
      validate={validate}
      {...props}
    />
  );
}
