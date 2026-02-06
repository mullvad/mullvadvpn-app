import { useCallback } from 'react';

import { messages } from '../../../../../../../../shared/gettext';
import log from '../../../../../../../../shared/logging';
import { useAppContext } from '../../../../../../../context';
import { Button } from '../../../../../../../lib/components';
import { useSelector } from '../../../../../../../redux/store';

export function DisconnectButton() {
  const { disconnectTunnel } = useAppContext();
  const tunnelState = useSelector((state) => state.connection.status.state);

  const onDisconnect = useCallback(async () => {
    try {
      await disconnectTunnel('gui-disconnect-button');
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to disconnect the tunnel: ${error.message}`);
    }
  }, [disconnectTunnel]);

  const displayAsCancel = tunnelState !== 'connected';

  return (
    <Button variant="destructive" onClick={onDisconnect}>
      <Button.Text>
        {displayAsCancel ? messages.gettext('Cancel') : messages.gettext('Disconnect')}
      </Button.Text>
    </Button>
  );
}
