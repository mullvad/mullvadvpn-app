import { messages } from '../../shared/gettext';
import { TunnelState } from '../../shared/daemon-rpc-types';
import { NotificationIndicatorType } from './notification';

export const nonBlockingError = {
  condition: (tunnelState: TunnelState) =>
    tunnelState.state === 'error' && !tunnelState.details.isBlocking,
  systemNotification: {
    message: messages.pgettext('notifications', 'Critical error (your attention is required)'),
    important: true,
  },
  inAppNotification: {
    indicator: 'error' as NotificationIndicatorType,
    title: messages.pgettext('in-app-notifications', 'YOU MIGHT BE LEAKING NETWORK TRAFFIC'),
    body: messages.pgettext(
      'in-app-notifications',
      'Failed to block all network traffic. Please troubleshoot or report the problem to us.',
    ),
  },
};
