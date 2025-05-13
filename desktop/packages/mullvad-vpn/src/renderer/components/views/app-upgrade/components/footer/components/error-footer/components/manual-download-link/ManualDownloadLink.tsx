import { messages } from '../../../../../../../../../../shared/gettext';
import { Button } from '../../../../../../../../../lib/components';
import { useHandleClick } from './hooks';

export function ManualDownloadLink() {
  const handleClick = useHandleClick();

  return (
    <Button
      aria-description={messages.pgettext('accessibility', 'Opens externally')}
      onClick={handleClick}>
      <Button.Text>
        {
          // TRANSLATORS: Button text to manually download the update
          messages.pgettext('app-upgrade-view', 'Manual download')
        }
      </Button.Text>
      <Button.Icon icon="external" />
    </Button>
  );
}
