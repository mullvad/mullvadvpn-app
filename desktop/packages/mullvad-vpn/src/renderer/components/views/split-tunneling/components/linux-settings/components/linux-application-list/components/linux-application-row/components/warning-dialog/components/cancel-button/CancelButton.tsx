import { messages } from '../../../../../../../../../../../../../../shared/gettext';
import { Dialog } from '../../../../../../../../../../../../../lib/components/dialog';
import { useDisabled } from '../../../../hooks';

export function CancelButton() {
  const disabled = useDisabled();

  return (
    <Dialog.CloseButton>
      <Dialog.CloseButton.Text>
        {disabled ? messages.gettext('Back') : messages.gettext('Cancel')}
      </Dialog.CloseButton.Text>
    </Dialog.CloseButton>
  );
}
