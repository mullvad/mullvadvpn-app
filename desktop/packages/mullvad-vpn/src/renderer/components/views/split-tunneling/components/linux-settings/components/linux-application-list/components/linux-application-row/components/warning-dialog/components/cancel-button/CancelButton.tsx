import { messages } from '../../../../../../../../../../../../../../shared/gettext';
import { Button } from '../../../../../../../../../../../../../lib/components';
import { useDisabled } from '../../../../hooks';
import { useHideWarningDialog } from '../../hooks';

export function CancelButton() {
  const disabled = useDisabled();
  const hideWarningDialog = useHideWarningDialog();

  return (
    <Button key="cancel" onClick={hideWarningDialog}>
      <Button.Text>{disabled ? messages.gettext('Back') : messages.gettext('Cancel')}</Button.Text>
    </Button>
  );
}
