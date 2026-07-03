import { messages } from '../../../../../shared/gettext';
import { Button } from '../../../../lib/components';
import type { ApiAccessMethodTestingState } from '../types';

export function getTestingDialogButtons(
  type: ApiAccessMethodTestingState,
  save: () => void,
  cancel: () => void,
) {
  const saveButton = (
    <Button key="confirm" onClick={save}>
      <Button.Text>{messages.gettext('Save')}</Button.Text>
    </Button>
  );
  const cancelButton = (
    <Button key="cancel" onClick={cancel}>
      <Button.Text>{messages.gettext('Cancel')}</Button.Text>
    </Button>
  );
  const disabledCancelButton = (
    <Button key="cancel" onClick={cancel} disabled>
      <Button.Text>{messages.gettext('Cancel')}</Button.Text>
    </Button>
  );

  switch (type) {
    case 'success':
      return [disabledCancelButton];
    case 'failure':
      return [saveButton, cancelButton];
    case 'testing':
      return [cancelButton];
    default:
      return type satisfies never;
  }
}
