import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import { Dialog } from '../../../../lib/components/dialog';
import type { ApiAccessMethodTestingState } from '../types';

export function getTestingDialogText(
  type: ApiAccessMethodTestingState,
  newMethod: boolean,
  name: string,
) {
  let subtitle: string | undefined = undefined;
  if (type === 'failure') {
    subtitle = newMethod
      ? sprintf(
          messages.pgettext(
            'api-access-methods-view',
            'The API could not be reached using the %(name)s method.',
          ),
          { name },
        )
      : sprintf(
          // TRANSLATORS: %(save)s - Will be replaced with the translation for the word "Save".
          messages.pgettext(
            'api-access-methods-view',
            'Clicking “%(save)s” changes the in use method.',
          ),
          { save: messages.gettext('Save') },
        );
  }
  return subtitle ? <Dialog.Text>{subtitle}</Dialog.Text> : undefined;
}
