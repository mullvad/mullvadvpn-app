import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import { NotificationIndicatorType } from './notification';

export const reconnecting = {
  condition: (tunnelState: TunnelState) =>
    tunnelState.state === 'disconnecting' && tunnelState.details === 'reconnect',
  systemNotification: {
    message: messages.pgettext('notifications', 'Reconnecting'),
    important: false,
  },
  inAppNotification: {
    indicator: 'error' as NotificationIndicatorType,
    title: messages.pgettext('in-app-notifications', 'BLOCKING INTERNET'),
  },
};
