import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';

export const disconnected = {
  condition: (tunnelState: TunnelState) => tunnelState.state === 'disconnected',
  systemNotification: {
    message: messages.pgettext('notifications', 'Unsecured'),
    important: false,
  },
};
