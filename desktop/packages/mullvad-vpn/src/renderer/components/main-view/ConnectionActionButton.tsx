import { useCallback } from 'react';

import { messages } from '../../../shared/gettext';
import log from '../../../shared/logging';
import { useAppContext } from '../../context';
import { Button } from '../../lib/components';
import { useSelector } from '../../redux/store';

export default function ConnectionActionButton() {
  const tunnelState = useSelector((state) => state.connection.status.state);

  if (tunnelState === 'disconnected' || tunnelState === 'disconnecting') {
    return <ConnectButton disabled={tunnelState === 'disconnecting'} />;
  } else {
    return <DisconnectButton />;
  }
}

function ConnectButton(props: Partial<Parameters<typeof Button>[0]>) {
  const { connectTunnel } = useAppContext();

  const onConnect = useCallback(async () => {
    try {
      await connectTunnel();
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to connect the tunnel: ${error.message}`);
    }
  }, [connectTunnel]);

  return (
    <Button variant="success" onClick={onConnect} {...props}>
      <Button.Text>{messages.pgettext('tunnel-control', 'Connect')}</Button.Text>
    </Button>
  );
}

function DisconnectButton() {
  const { disconnectTunnel } = useAppContext();
  const tunnelState = useSelector((state) => state.connection.status.state);

  const onDisconnect = useCallback(async () => {
    try {
      await disconnectTunnel();
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
