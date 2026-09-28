import { messages } from '../../../../../../../../../../../shared/gettext';
import { useTextFieldItemContext } from '../../../TextFieldItemContext';

export function useFilterButtonLabel() {
  const { id } = useTextFieldItemContext();

  switch (id) {
    case 'entry':
      // TRANSLATORS: Label provided to accessibility tools such as screenreaders to inform the user
      // that they can click the button to configure filters for the entry location.
      return messages.pgettext(
        'accessibility',
        'Click button to configure filters for entry location',
      );
    case 'entryAutomatic':
      // TRANSLATORS: Label provided to accessibility tools such as screenreaders to inform the user
      // that entry location filters are disabled due to the use of the Automatic entry location.
      return messages.pgettext(
        'accessibility',
        'Entry location filters are overridden due to use of Automatic entry location',
      );
    case 'exit':
      // TRANSLATORS: Label provided to accessibility tools such as screenreaders to inform the user
      // that they can click the button to configure filters for the exit location.
      return messages.pgettext(
        'accessibility',
        'Click button to configure filters for exit location',
      );
    default:
      return id satisfies never;
  }
}
