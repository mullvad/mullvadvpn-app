import { sprintf } from 'sprintf-js';

import { strings } from '../../config.json';
import {
  AuthFailedError,
  ErrorState,
  ErrorStateCause,
  TunnelParameterError,
  TunnelState,
} from '../daemon-rpc-types';
import { messages } from '../gettext';
import {
  InAppNotification,
  InAppNotificationAction,
  InAppNotificationProvider,
  SystemNotification,
  SystemNotificationCategory,
  SystemNotificationProvider,
  SystemNotificationSeverityType,
} from './notification';

interface ErrorNotificationContext {
  tunnelState: TunnelState;
  hasExcludedApps: boolean;
}

export class ErrorNotificationProvider
  implements SystemNotificationProvider, InAppNotificationProvider {
  public constructor(private context: ErrorNotificationContext) {}

  public mayDisplay = () => this.context.tunnelState.state === 'error';

  public getSystemNotification(): SystemNotification | undefined {
    if (this.context.tunnelState.state === 'error') {
      let message = getMessage(this.context.tunnelState.details);
      if (!this.context.tunnelState.details.blockingError && this.context.hasExcludedApps) {
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
        severity:
          this.context.tunnelState.details.blockingError === undefined
            ? SystemNotificationSeverityType.low
            : SystemNotificationSeverityType.high,
        category: SystemNotificationCategory.tunnelState,
      };
    } else {
      return undefined;
    }
  }

  public getInAppNotification(): InAppNotification | undefined {
    if (this.context.tunnelState.state === 'error') {
      let subtitle = getMessage(this.context.tunnelState.details);
      if (!this.context.tunnelState.details.blockingError && this.context.hasExcludedApps) {
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
          this.context.tunnelState.details.cause === ErrorStateCause.isOffline
            ? 'warning'
            : 'error',
        title: this.context.tunnelState.details.blockingError
          ? messages.pgettext('in-app-notifications', 'NETWORK TRAFFIC MIGHT BE LEAKING')
          : messages.pgettext('in-app-notifications', 'BLOCKING INTERNET'),
        subtitle,
        action: getActions(this.context.tunnelState.details) ?? undefined,
      };
    } else {
      return undefined;
    }
  }
}

function getMessage(errorState: ErrorState): string {
  if (errorState.blockingError) {
    if (errorState.cause === ErrorStateCause.setFirewallPolicyError) {
      switch (process.platform ?? window.env.platform) {
        case 'win32':
          return messages.pgettext(
            'notifications',
            'Unable to block all network traffic. Try temporarily disabling any third-party antivirus or security software or send a problem report.',
          );
        case 'linux':
          return messages.pgettext(
            'notifications',
            'Unable to block all network traffic. Try updating your kernel or send a problem report.',
          );
      }
    }

    return messages.pgettext(
      'notifications',
      'Unable to block all network traffic. Please troubleshoot or send a problem report.',
    );
  } else {
    switch (errorState.cause) {
      case ErrorStateCause.authFailed:
        switch (errorState.authFailedError) {
          case AuthFailedError.invalidAccount:
            return messages.pgettext(
              'auth-failure',
              'You are logged in with an invalid account number. Please log out and try another one.',
            );

          case AuthFailedError.expiredAccount:
            return messages.pgettext('auth-failure', 'Blocking internet: account is out of time');

          case AuthFailedError.tooManyConnections:
            return messages.pgettext(
              'auth-failure',
              'Too many simultaneous connections on this account. Disconnect another device or try connecting again shortly.',
            );

          case AuthFailedError.unknown:
          default:
            return messages.pgettext(
              'auth-failure',
              'Unable to authenticate account. Please send a problem report.',
            );
        }
      case ErrorStateCause.ipv6Unavailable:
        return messages.pgettext(
          'notifications',
          'Could not configure IPv6. Disable it in the app or enable it on your device.',
        );
      case ErrorStateCause.setFirewallPolicyError:
        switch (process.platform ?? window.env.platform) {
          case 'win32':
            return messages.pgettext(
              'notifications',
              'Unable to apply firewall rules. Try temporarily disabling any third-party antivirus or security software.',
            );
          case 'linux':
            return messages.pgettext(
              'notifications',
              'Unable to apply firewall rules. Try updating your kernel.',
            );
          default:
            return messages.pgettext('notifications', 'Unable to apply firewall rules.');
        }
      case ErrorStateCause.setDnsError:
        return messages.pgettext(
          'notifications',
          'Unable to set system DNS server. Please send a problem report.',
        );
      case ErrorStateCause.startTunnelError:
        return messages.pgettext(
          'notifications',
          'Unable to start tunnel connection. Please send a problem report.',
        );
      case ErrorStateCause.tunnelParameterError:
        return getTunnelParameterMessage(errorState.parameterError);
      case ErrorStateCause.isOffline:
        return messages.pgettext(
          'notifications',
          'Your device is offline. The tunnel will automatically connect once your device is back online.',
        );
      case ErrorStateCause.splitTunnelError:
        return messages.pgettext(
          'notifications',
          'Unable to communicate with Mullvad kernel driver. Try reconnecting or send a problem report.',
        );
    }
  }
}

function getTunnelParameterMessage(error: TunnelParameterError): string {
  switch (error) {
    /// TODO: once bridge constraints can be set, add a more descriptive error message
    case TunnelParameterError.noMatchingBridgeRelay:
    case TunnelParameterError.noMatchingRelay:
      return messages.pgettext(
        'notifications',
        'No servers match your settings, try changing server or other settings.',
      );
    case TunnelParameterError.noWireguardKey:
      return sprintf(
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(wireguard)s - will be replaced with "WireGuard"
        messages.pgettext(
          'notifications',
          'Valid %(wireguard)s key is missing. Manage keys under Advanced settings.',
        ),
        { wireguard: strings.wireguard },
      );
    case TunnelParameterError.customTunnelHostResolutionError:
      return messages.pgettext(
        'notifications',
        'Unable to resolve host of custom tunnel. Try changing your settings.',
      );
  }
}

function getActions(errorState: ErrorState): InAppNotificationAction | void {
  const platform = process.platform ?? window.env.platform;

  if (errorState.cause === ErrorStateCause.setFirewallPolicyError && platform === 'linux') {
    return {
      type: 'troubleshoot-dialog',
      troubleshoot: {
        details: messages.pgettext(
          'troubleshoot',
          'This can happen because the kernel is old, or if you have removed a kernel.',
        ),
        steps: [
          messages.pgettext('troubleshoot', 'Update your kernel.'),
          messages.pgettext('troubleshoot', 'Make sure you have NF tables support.'),
        ],
      },
    };
  } else if (errorState.cause === ErrorStateCause.setDnsError) {
    const troubleshootSteps = [];
    if (platform === 'darwin') {
      troubleshootSteps.push(
        messages.pgettext(
          'troubleshoot',
          'Try to turn Wi-Fi Calling off in the FaceTime app settings and restart the Mac.',
        ),
        messages.pgettext(
          'troubleshoot',
          'Uninstall or disable other DNS, networking and ads/website blocking apps.',
        ),
      );
    } else if (platform === 'win32') {
      troubleshootSteps.push(
        messages.pgettext(
          'troubleshoot',
          'Uninstall or disable other DNS, networking and ads/website blocking apps.',
        ),
      );
    }

    return {
      type: 'troubleshoot-dialog',
      troubleshoot: {
        details: messages.pgettext(
          'troubleshoot',
          'This error can happen when something other than Mullvad is actively updating the DNS.',
        ),
        steps: troubleshootSteps,
      },
    };
  } else if (errorState.cause === ErrorStateCause.splitTunnelError) {
    return {
      type: 'troubleshoot-dialog',
      troubleshoot: {
        details: messages.pgettext(
          'troubleshoot',
          'Unable to communicate with Mullvad kernel driver.',
        ),
        steps: [
          messages.pgettext('troubleshoot', 'Try reconnecting.'),
          messages.pgettext('troubleshoot', 'Try restarting your device.'),
        ],
      },
    };
  }
}
