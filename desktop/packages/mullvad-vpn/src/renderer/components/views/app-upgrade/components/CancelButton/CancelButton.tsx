import { messages } from '../../../../../../shared/gettext';
import { Button } from '../../../../../lib/components';
import { useDisabled, useHandleOnClick } from './hooks';

export function CancelButton() {
  const disabled = useDisabled();
  const handleOnClick = useHandleOnClick();

  return (
    <Button disabled={disabled} onClick={handleOnClick}>
      <Button.Text>
        {
          // TRANSLATORS: Button text to cancel the download of an update
          messages.pgettext('app-upgrade-view', 'Cancel')
        }
      </Button.Text>
    </Button>
  );
}
