import styled from 'styled-components';

import { TunnelState } from '../../../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../../../shared/gettext';
import { colors } from '../../../../../../../lib/foundations';
import { useSelector } from '../../../../../../../redux/store';
import { largeText } from '../../../../../../common-styles';

const StyledConnectionStatus = styled.span<{ $color: string }>(largeText, (props) => ({
  minHeight: '24px',
  color: props.$color,
  marginBottom: '4px',
}));

export function ConnectionStatus() {
  const tunnelState = useSelector((state) => state.connection.status);

  const color = getConnectionSTatusLabelColor(tunnelState);
  const text = getConnectionStatusLabelText(tunnelState);

  return (
    <StyledConnectionStatus role="status" $color={color}>
      {text}
    </StyledConnectionStatus>
  );
}

function getConnectionSTatusLabelColor(tunnelState: TunnelState) {
  switch (tunnelState.state) {
    case 'connected':
      return colors.green;
    case 'connecting':
    case 'disconnecting':
      return colors.white;
    case 'disconnected':
      return tunnelState.lockedDown ? colors.white : colors.red;
    case 'error':
      return tunnelState.details.blockingError ? colors.red : colors.white;
  }
}

function getConnectionStatusLabelText(tunnelState: TunnelState) {
  switch (tunnelState.state) {
    case 'connected':
      return messages.gettext('CONNECTED');
    case 'connecting':
      return messages.gettext('CONNECTING...');
    case 'disconnecting':
      return messages.gettext('DISCONNECTING...');
    case 'disconnected':
      return messages.gettext('DISCONNECTED');
    case 'error':
      return tunnelState.details.blockingError
        ? messages.gettext('FAILED TO SECURE CONNECTION')
        : messages.gettext('BLOCKED CONNECTION');
  }
}
