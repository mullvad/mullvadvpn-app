import { messages } from '../../../../../../../shared/gettext';
import { SettingsListbox } from '../../../../../../components/settings-listbox';

export type AlwaysOptionProps = {
  disabled: boolean;
};

export function AlwaysOption({ disabled }: AlwaysOptionProps) {
  return (
    <SettingsListbox.Options.BaseOption value="always" disabled={disabled}>
      {
        // TRANSLATORS: Label for the Multihop mode setting's option to always use the multihop
        // TRANSLATORS: feature.
        messages.pgettext('wireguard-settings-view', 'Always')
      }
    </SettingsListbox.Options.BaseOption>
  );
}
