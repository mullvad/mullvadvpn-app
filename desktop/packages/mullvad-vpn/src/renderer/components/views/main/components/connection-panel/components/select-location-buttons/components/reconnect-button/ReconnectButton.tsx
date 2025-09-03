import { useCallback } from 'react';
import styled from 'styled-components';

import { messages } from '../../../../../../../../../../shared/gettext';
import log from '../../../../../../../../../../shared/logging';
import { useAppContext } from '../../../../../../../../../context';
import { Button, ButtonProps, Icon } from '../../../../../../../../../lib/components';

const StyledReconnectButton = styled(Button)({
  minWidth: '40px',
});

export function ReconnectButton(props: ButtonProps) {
  const { reconnectTunnel } = useAppContext();

  const onReconnect = useCallback(async () => {
    try {
      await reconnectTunnel();
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to reconnect the tunnel: ${error.message}`);
    }
  }, [reconnectTunnel]);

  return (
    <StyledReconnectButton
      onClick={onReconnect}
      width="fit"
      aria-label={messages.gettext('Reconnect')}
      {...props}>
      <Icon icon="reconnect" />
    </StyledReconnectButton>
  );
}
