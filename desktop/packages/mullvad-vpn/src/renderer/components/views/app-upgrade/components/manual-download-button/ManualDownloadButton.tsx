import { messages } from '../../../../../../shared/gettext';
import { Button, Icon } from '../../../../../lib/components';
import { useDisabled, useHandleClick } from './hooks';

export function ManualDownloadButton() {
  const disabled = useDisabled();
  const handleClick = useHandleClick();

  return (
    <Button
      aria-description={messages.pgettext('accessibility', 'Opens externally')}
      disabled={disabled}
      onClick={handleClick}>
      <Button.Text>
        {
          // TRANSLATORS: Button text to manually download the update
          messages.pgettext('app-upgrade-view', 'Manual download')
        }
      </Button.Text>
      <Icon icon="external" />
    </Button>
  );
}
