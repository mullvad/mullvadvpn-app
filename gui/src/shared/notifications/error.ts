import { parseAuthFailure } from '../../renderer/lib/auth-failure';
import { messages } from '../gettext';
import { TunnelState, TunnelParameterError } from '../daemon-rpc-types';
import { NotificationIndicatorType } from './notification';

export const error = {
  condition: (tunnelState: TunnelState) => tunnelState.state === 'error',
  systemNotification: {
    message: getSystemNotificationMessage,
    important: false,
  },
  inAppNotification: {
    indicator: 'error' as NotificationIndicatorType,
    title: messages.pgettext('in-app-notifications', 'BLOCKING INTERNET'),
    body: getInAppNotificationBody,
  },
};

function getSystemNotificationMessage(tunnelState: TunnelState) {
  if (
    tunnelState.state === 'error' &&
    tunnelState.details.isBlocking &&
    tunnelState.details.cause.reason === 'tunnel_parameter_error' &&
    tunnelState.details.cause.details === 'no_wireguard_key'
  ) {
    return messages.pgettext('notifications', 'Blocking internet: Valid WireGuard key is missing');
  } else {
    return messages.pgettext('notifications', 'Blocking internet');
  }
}

function getInAppNotificationBody(tunnelState: TunnelState) {
  if (tunnelState.state === 'error') {
    const blockReason = tunnelState.details.cause;
    switch (blockReason.reason) {
      case 'auth_failed':
        return parseAuthFailure(blockReason.details).message;
      case 'ipv6_unavailable':
        return messages.pgettext(
          'in-app-notifications',
          'Could not configure IPv6, please enable it on your system or disable it in the app',
        );
      case 'set_firewall_policy_error': {
        let extraMessage = null;
        switch (process.platform) {
          case 'linux':
            extraMessage = messages.pgettext('in-app-notifications', 'Your kernel may be outdated');
            break;
          case 'win32':
            extraMessage = messages.pgettext(
              'in-app-notifications',
              'This might be caused by third party security software',
            );
            break;
        }
        return `${messages.pgettext(
          'in-app-notifications',
          'Failed to apply firewall rules. The device might currently be unsecured',
        )}${extraMessage ? '. ' + extraMessage : ''}`;
      }
      case 'set_dns_error':
        return messages.pgettext('in-app-notifications', 'Failed to set system DNS server');
      case 'start_tunnel_error':
        return messages.pgettext('in-app-notifications', 'Failed to start tunnel connection');
      case 'tunnel_parameter_error':
        return getTunnelParameterMessage(blockReason.details);
      case 'is_offline':
        return messages.pgettext(
          'in-app-notifications',
          'This device is offline, no tunnels can be established',
        );
      case 'tap_adapter_problem':
        return messages.pgettext(
          'in-app-notifications',
          "Unable to detect a working TAP adapter on this device. If you've disabled it, enable it again. Otherwise, please reinstall the app",
        );
    }
  } else {
    return '';
  }
}

function getTunnelParameterMessage(err: TunnelParameterError): string {
  switch (err) {
    /// TODO: once bridge constraints can be set, add a more descriptive error message
    case 'no_matching_bridge_relay':
    case 'no_matching_relay':
      return messages.pgettext(
        'in-app-notifications',
        'No relay server matches the current settings. You can try changing the location or the relay settings.',
      );
    case 'no_wireguard_key':
      return messages.pgettext(
        'in-app-notifications',
        'Valid WireGuard key is missing. Manage keys under Advanced settings.',
      );
    case 'custom_tunnel_host_resultion_error':
      return messages.pgettext(
        'in-app-notifications',
        'Failed to resolve host of custom tunnel. Consider changing the settings',
      );
  }
}
