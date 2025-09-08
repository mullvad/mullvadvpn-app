import { messages } from '../../../../../../../../shared/gettext';
import { Button } from '../../../../../../../lib/components';
import { useDisabled, useLaunchWithFilePicker } from './hooks';

export function OpenFilePickerButton() {
  const disabled = useDisabled();
  const launchWithFilePicker = useLaunchWithFilePicker();

  return (
    <Button disabled={disabled} onClick={launchWithFilePicker}>
      <Button.Text>
        {
          // TRANSLATORS: Button label for browsing applications with split tunneling.
          messages.pgettext('split-tunneling-view', 'Find another app')
        }
      </Button.Text>
    </Button>
  );
}
