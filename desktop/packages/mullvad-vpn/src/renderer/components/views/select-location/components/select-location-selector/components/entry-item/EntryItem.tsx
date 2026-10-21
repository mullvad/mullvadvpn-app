import { messages } from '../../../../../../../../shared/gettext';
import { useEntryLocation } from '../../../../../../../features/locations/hooks';
import { LocationType } from '../../../../../../../features/locations/types';
import { useGetIsLocationValid, useLocationName } from '../../hooks';
import { type SelectLocationSelectorItemProps, TextFieldItem } from '../text-field-item';

export type EntryItemProps = Omit<
  SelectLocationSelectorItemProps,
  'name' | 'placeholder' | 'value' | 'id' | 'type'
>;

export function EntryItem(props: EntryItemProps) {
  const { entryLocation } = useEntryLocation();
  const defaultValue = useLocationName(entryLocation);
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
