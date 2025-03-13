import { messages } from '../../../../../../../../shared/gettext';
import { Button } from '../../../../../../../lib/components';
import { useDisabled, useHandleOnClick } from './hooks';

export function CancelButton() {
  const handleOnClick = useHandleOnClick();
  const disabled = useDisabled();

  return (
    <Button disabled={disabled} onClick={handleOnClick}>
      <Button.Text>
        {
          // TRANSLATORS: Button text to cancel the download of an update
          messages.pgettext('download-update-view', 'Cancel')
        }
      </Button.Text>
    </Button>
  );
}
