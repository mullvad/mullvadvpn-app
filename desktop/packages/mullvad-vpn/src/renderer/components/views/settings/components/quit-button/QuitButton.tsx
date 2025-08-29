import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { Button } from '../../../../../lib/components';
import { useIsConnected } from './hooks';

export function QuitButton() {
  const { quit } = useAppContext();
  const isConnected = useIsConnected();

  return (
    <Button variant="destructive" onClick={quit}>
      <Button.Text>
        {isConnected ? messages.gettext('Disconnect & quit') : messages.gettext('Quit')}
      </Button.Text>
    </Button>
  );
}
