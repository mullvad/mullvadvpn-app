import { sprintf } from 'sprintf-js';

import { strings } from '../../config.json';
import { hasExpired } from '../account-expiry';
import { AuthFailureKind, parseAuthFailure } from '../auth-failure';
import { IErrorState, TunnelParameterError, TunnelState } from '../daemon-rpc-types';
import { messages } from '../gettext';
import {
  InAppNotification,
  InAppNotificationProvider,
  SystemNotificationProvider,
} from './notification';

interface ErrorNotificationContext {
  tunnelState: TunnelState;
  accountExpiry?: string;
  hasExcludedApps: boolean;
}

export class ErrorNotificationProvider
  implements SystemNotificationProvider, InAppNotificationProvider {
  public constructor(private context: ErrorNotificationContext) {}

  public mayDisplay = () => this.context.tunnelState.state === 'error';

  public getSystemNotification() {
    if (this.context.tunnelState.state === 'error') {
      let message = getMessage(this.context.tunnelState.details, this.context.accountExpiry);
      if (!this.context.tunnelState.details.blockFailure && this.context.hasExcludedApps) {
        message = `${message} ${sprintf(
          messages.pgettext(
            'notifications',
            'The apps excluded with %(splitTunneling)s might not work properly right now.',
          ),
          { splitTunneling: strings.splitTunneling.toLowerCase() },
        )}`;
      }

      return {
        message,
        critical: !!this.context.tunnelState.details.blockFailure,
      };
    } else {
      return undefined;
    }
  }

  public getInAppNotification(): InAppNotification | undefined {
    if (this.context.tunnelState.state === 'error') {
      let subtitle = getMessage(this.context.tunnelState.details, this.context.accountExpiry);
      if (!this.context.tunnelState.details.blockFailure && this.context.hasExcludedApps) {
        subtitle = `${subtitle} ${sprintf(
          messages.pgettext(
            'notifications',
            'The apps excluded with %(splitTunneling)s might not work properly right now.',
          ),
          { splitTunneling: strings.splitTunneling.toLowerCase() },
        )}`;
      }

      return {
        indicator:
          this.context.tunnelState.details.cause.reason === 'is_offline' ? 'warning' : 'error',
        title: !this.context.tunnelState.details.blockFailure
          ? messages.pgettext('in-app-notifications', 'BLOCKING INTERNET')
          : messages.pgettext('in-app-notifications', 'NETWORK TRAFFIC MIGHT BE LEAKING'),
        subtitle,
      };
    } else {
      return undefined;
    }
  }
}

function getMessage(errorDetails: IErrorState, accountExpiry?: string): string {
  if (errorDetails.blockFailure) {
    if (errorDetails.cause.reason === 'set_firewall_policy_error') {
      switch (process.platform ?? window.env.platform) {
        case 'win32':
          return messages.pgettext(
            'notifications',
            'Unable to block all network traffic. Try disabling any third-party antivirus or security software or contact support.',
          );
        case 'linux':
          return messages.pgettext(
            'notifications',
            'Unable to block all network traffic. Try updating your kernel or contact support.',
          );
      }
    }

    return messages.pgettext(
      'notifications',
      'Unable to block all network traffic. Please troubleshoot or contact support.',
    );
  } else {
    switch (errorDetails.cause.reason) {
      case 'auth_failed': {
        const authFailure = parseAuthFailure(errorDetails.cause.details);
        if (
          authFailure.kind === AuthFailureKind.unknown &&
          accountExpiry &&
          hasExpired(accountExpiry)
        ) {
          return messages.pgettext(
            'auth-failure',
            'You are logged in with an invalid account number. Please log out and try another one.',
          );
        } else {
          return authFailure.message;
        }
      }
      case 'ipv6_unavailable':
        return messages.pgettext(
          'notifications',
          'Could not configure IPv6. Disable it in the app or enable it on your device.',
        );
      case 'set_firewall_policy_error':
        switch (process.platform ?? window.env.platform) {
          case 'win32':
            return messages.pgettext(
              'notifications',
              'Unable to apply firewall rules. Try disabling any third-party antivirus or security software.',
            );
          case 'linux':
            return messages.pgettext(
              'notifications',
              'Unable to apply firewall rules. Try updating your kernel.',
            );
          default:
            return messages.pgettext('notifications', 'Unable to apply firewall rules.');
        }
      case 'set_dns_error':
        return messages.pgettext(
          'notifications',
          'Unable to set system DNS server. Please contact support.',
        );
      case 'start_tunnel_error':
        return messages.pgettext(
          'notifications',
          'Unable to start tunnel connection. Please contact support.',
        );
      case 'tunnel_parameter_error':
        return getTunnelParameterMessage(errorDetails.cause.details);
      case 'is_offline':
        return messages.pgettext(
          'notifications',
          'Your device is offline. The tunnel will automatically connect once your device is back online.',
        );
      case 'split_tunnel_error':
        return messages.pgettext(
          'notifications',
          'Unable to communicate with Mullvad kernel driver. Try reconnecting or contact support.',
        );
    }
  }
}

function getTunnelParameterMessage(err: TunnelParameterError): string {
  switch (err) {
    /// TODO: once bridge constraints can be set, add a more descriptive error message
    case 'no_matching_bridge_relay':
    case 'no_matching_relay':
      return messages.pgettext(
        'notifications',
        'No servers in your selected location match your settings.',
      );
    case 'no_wireguard_key':
      return sprintf(
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(wireguard)s - will be replaced with "WireGuard"
        messages.pgettext(
          'notifications',
          'Valid %(wireguard)s key is missing. Manage keys under Advanced settings.',
        ),
        { wireguard: strings.wireguard },
      );
    case 'custom_tunnel_host_resultion_error':
      return messages.pgettext(
        'notifications',
        'Unable to resolve host of custom tunnel. Try changing your settings.',
      );
  }
}
