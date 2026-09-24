import { messages } from '../../../../../../../../shared/gettext';
import { useSelectedLocations } from '../../../../../../../features/locations/hooks';
import { useLocationName } from '../../hooks';
import { type SelectLocationSelectorItemProps, TextFieldItem } from '../text-field-item';

export type EntryItemProps = Omit<
  SelectLocationSelectorItemProps,
  'name' | 'placeholder' | 'value' | 'id' | 'type'
>;

export function EntryItem(props: EntryItemProps) {
  const { entry } = useSelectedLocations();
  const defaultValue = useLocationName(entry);

  // TODO: Look up selected entry hostname and check if it is in the filteredRelays list.
  const invalid = true;

  return (
    <TextFieldItem
      id="entry"
      type="entry"
      aria-label={messages.gettext('Search entry location or server, press enter to search')}
      placeholder={messages.gettext('Search entry location or server')}
      defaultValue={defaultValue}
      invalid={invalid}
      {...props}
    />
  );
}
