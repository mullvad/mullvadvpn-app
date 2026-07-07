import { messages } from '../../../../../../../shared/gettext';
import { useMultihop } from '../../../../../../features/multihop/hooks';

export const useMessage = () => {
  const { multihop } = useMultihop();

  switch (multihop) {
    case 'always':
      // TRANSLATORS: Display name of the currently selected option for the Multihop mode setting
      // TRANSLATORS: when the "Always" option is selected.
      return messages.pgettext('settings-view', 'Always');
    case 'never':
      // TRANSLATORS: Display name of the currently selected option for the Multihop mode setting
      // TRANSLATORS: when the "Never" option is selected.
      return messages.pgettext('settings-view', 'Never');
    case 'when-needed':
      // TRANSLATORS: Display name of the currently selected option for the Multihop mode setting
      // TRANSLATORS: when the "When needed" option is selected.
      return messages.pgettext('settings-view', 'When needed');
    default:
      return multihop satisfies never;
  }
};
