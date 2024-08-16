import { useCallback } from 'react';
import styled from 'styled-components';

import { messages } from '../../../shared/gettext';
import log from '../../../shared/logging';
import { useAppContext } from '../../context';
import { useSelector } from '../../redux/store';
import { SmallButton, SmallButtonColor } from '../SmallButton';

const StyledConnectionButton = styled(SmallButton)({
  margin: 0,
});

export default function ConnectionActionButton() {
  const tunnelState = useSelector((state) => state.connection.status.state);

  if (tunnelState === 'disconnected') {
    return <ConnectButton />;
  } else {
    return <DisconnectButton disabled={tunnelState === 'disconnecting'} />;
  }
}

function ConnectButton() {
  const { connectTunnel } = useAppContext();

  const onConnect = useCallback(async () => {
    try {
      await connectTunnel();
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to connect the tunnel: ${error.message}`);
    }
  }, []);

  return (
    <StyledConnectionButton color={SmallButtonColor.green} onClick={onConnect}>
      {messages.pgettext('tunnel-control', 'Connect')}
    </StyledConnectionButton>
  );
}

function DisconnectButton(props: Partial<Parameters<typeof SmallButton>[0]>) {
  const { disconnectTunnel } = useAppContext();
  const tunnelState = useSelector((state) => state.connection.status.state);

  const onDisconnect = useCallback(async () => {
    try {
      await disconnectTunnel();
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to disconnect the tunnel: ${error.message}`);
    }
  }, []);

  const displayAsCancel = tunnelState !== 'connected';

  return (
    <StyledConnectionButton color={SmallButtonColor.red} onClick={onDisconnect} {...props}>
      {displayAsCancel ? messages.gettext('Cancel') : messages.gettext('Disconnect')}
    </StyledConnectionButton>
  );
}
