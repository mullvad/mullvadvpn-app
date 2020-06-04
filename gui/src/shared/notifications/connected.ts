import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';

export const connectedTo = {
  condition: (tunnelState: TunnelState) =>
    tunnelState.state === 'connected' &&
    tunnelState.details &&
    tunnelState.details.location &&
    tunnelState.details.location.hostname,
  systemNotification: {
    message: messages.pgettext('notifications', 'Connected to %(location)s'),
    important: false,
  },
};

export const connected = {
  condition: (tunnelState: TunnelState) => tunnelState.state === 'connected',
  systemNotification: {
    message: messages.pgettext('notifications', 'Secured'),
    important: false,
  },
};
