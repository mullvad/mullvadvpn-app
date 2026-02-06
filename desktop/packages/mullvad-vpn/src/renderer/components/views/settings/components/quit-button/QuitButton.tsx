import { useCallback } from 'react';

import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { Button } from '../../../../../lib/components';
import { useIsConnected } from './hooks';

export function QuitButton() {
  const { quit } = useAppContext();
  const isConnected = useIsConnected();

  const handleClick = useCallback(() => {
    quit('gui-quit-button');
  }, [quit]);

  return (
    <Button variant="destructive" onClick={handleClick}>
      <Button.Text>
        {isConnected ? messages.gettext('Disconnect & quit') : messages.gettext('Quit')}
      </Button.Text>
    </Button>
  );
}
