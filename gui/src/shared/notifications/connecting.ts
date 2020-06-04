import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import { NotificationIndicatorType } from './notification';

export const connectingTo = {
  condition: (tunnelState: TunnelState) =>
    tunnelState.state === 'connecting' &&
    tunnelState.details &&
    tunnelState.details.location &&
    tunnelState.details.location.hostname,
  systemNotification: {
    message: messages.pgettext('notifications', 'Connecting to %(location)s'),
    important: false,
  },
  inAppNotification: {
    indicator: 'error' as NotificationIndicatorType,
    title: messages.pgettext('in-app-notifications', 'BLOCKING INTERNET'),
  },
};

export const connecting = {
  condition: (tunnelState: TunnelState) => tunnelState.state === 'connecting',
  systemNotification: {
    message: messages.pgettext('notifications', 'Connecting'),
    important: false,
  },
  inAppNotification: {
    indicator: 'error' as NotificationIndicatorType,
    title: messages.pgettext('in-app-notifications', 'BLOCKING INTERNET'),
  },
};
