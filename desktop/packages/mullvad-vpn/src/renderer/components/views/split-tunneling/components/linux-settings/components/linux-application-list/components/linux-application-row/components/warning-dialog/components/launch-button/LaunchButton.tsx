import { messages } from '../../../../../../../../../../../../../../shared/gettext';
import { Dialog } from '../../../../../../../../../../../../../lib/components/dialog';
import { useHandleClick } from './hooks';

export function LaunchButton() {
  const handleClick = useHandleClick();

  return (
    <Dialog.Button onClick={handleClick}>
      <Dialog.Button.Text>
        {
          // TRANSLATORS: Button label for launching an application with split tunneling.
          messages.pgettext('split-tunneling-view', 'Launch')
        }
      </Dialog.Button.Text>
    </Dialog.Button>
  );
}
