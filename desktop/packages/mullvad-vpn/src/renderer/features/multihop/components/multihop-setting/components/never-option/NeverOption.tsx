import { messages } from '../../../../../../../shared/gettext';
import { SettingsListbox } from '../../../../../../components/settings-listbox';

export type NeverOptionProps = {
  disabled: boolean;
};

export function NeverOption({ disabled }: NeverOptionProps) {
  return (
    <SettingsListbox.Options.BaseOption value="never" disabled={disabled}>
      {
        // TRANSLATORS: Label for the Multihop mode setting's option to never use the multihop
        // TRANSLATORS: feature.
        messages.pgettext('wireguard-settings-view', 'Never')
      }
    </SettingsListbox.Options.BaseOption>
  );
}
