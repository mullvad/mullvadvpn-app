import { messages } from '../../../../../../../../../../../../../../shared/gettext';
import { Button } from '../../../../../../../../../../../../../lib/components';
import { useHandleClick } from './hooks';

export function LaunchButton() {
  const handleClick = useHandleClick();

  return (
    <Button onClick={handleClick}>
      <Button.Text>
        {
          // TRANSLATORS: Button label for launching an application with split tunneling.
          messages.pgettext('split-tunneling-view', 'Launch')
        }
      </Button.Text>
    </Button>
  );
}
