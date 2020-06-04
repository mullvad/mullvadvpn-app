import { messages } from '../../shared/gettext';
import { TunnelState } from '../daemon-rpc-types';
import { NotificationIndicatorType } from './notification';

export const blockWhenDisconnected = {
  condition: (tunnelState: TunnelState, blockWhenDisconnected: boolean) =>
    tunnelState.state === 'disconnected' && blockWhenDisconnected,
  inAppNotification: {
    indicator: 'error' as NotificationIndicatorType,
    title: messages.pgettext('in-app-notifications', 'BLOCKING INTERNET'),
    body: messages.pgettext('in-app-notifications', '"Always require VPN" is enabled.'),
  },
};
