import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { Button } from '../../../../../lib/components';
import { useSelector } from '../../../../../redux/store';

export function QuitButton() {
  const { quit } = useAppContext();
  const tunnelState = useSelector((state) => state.connection.status);

  return (
    <Button variant="destructive" onClick={quit}>
      <Button.Text>
        {tunnelState.state === 'disconnected'
          ? messages.gettext('Quit')
          : messages.gettext('Disconnect & quit')}
      </Button.Text>
    </Button>
  );
}
