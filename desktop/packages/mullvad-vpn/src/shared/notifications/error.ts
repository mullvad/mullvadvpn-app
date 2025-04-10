import { sprintf } from 'sprintf-js';

import { strings } from '../constants';
import {
  AuthFailedError,
  ErrorStateCause,
  ErrorStateDetails,
  TunnelParameterError,
  TunnelState,
} from '../daemon-rpc-types';
import { messages } from '../gettext';
import {
  InAppNotification,
  InAppNotificationAction,
  InAppNotificationProvider,
  InAppNotificationTroubleshootButton,
  SystemNotification,
  SystemNotificationCategory,
  SystemNotificationProvider,
  SystemNotificationSeverityType,
} from './notification';

interface ErrorNotificationContext {
  tunnelState: TunnelState;
  hasExcludedApps: boolean;
  showFullDiskAccessSettings?: () => void;
  disableSplitTunneling?: () => void;
}

export class ErrorNotificationProvider
  implements SystemNotificationProvider, InAppNotificationProvider
{
  public constructor(private context: ErrorNotificationContext) {}

  public mayDisplay = () => this.context.tunnelState.state === 'error';

  public getSystemNotification(): SystemNotification | undefined {
    if (this.context.tunnelState.state === 'error') {
      let message = this.getMessage(this.context.tunnelState.details);
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
      let subtitle = this.getMessage(this.context.tunnelState.details);
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
        action: this.getActions(this.context.tunnelState.details) ?? undefined,
      };
    } else {
      return undefined;
    }
  }

  private getMessage(errorState: ErrorStateDetails): string {
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
        case ErrorStateCause.createTunnelDeviceError:
          if (errorState.osError === 4319) {
            return messages.pgettext(
              'notifications',
              'Unable to start tunnel connection. This could be because of conflicts with VMware, please troubleshoot.',
            );
          }

          return messages.pgettext(
            'notifications',
            'Unable to start tunnel connection. Please send a problem report.',
          );
        case ErrorStateCause.tunnelParameterError:
          return this.getTunnelParameterMessage(errorState.parameterError);
        case ErrorStateCause.isOffline:
          return messages.pgettext(
            'notifications',
            'Your device is offline. The tunnel will automatically connect once your device is back online.',
          );
        case ErrorStateCause.needFullDiskPermissions:
          return messages.pgettext('notifications', 'Failed to enable split tunneling.');
        case ErrorStateCause.splitTunnelError:
          switch (process.platform ?? window.env.platform) {
            case 'darwin':
              return messages.pgettext(
                'notifications',
                'Failed to enable split tunneling. Please try reconnecting or disable split tunneling.',
              );
            default:
              return messages.pgettext(
                'notifications',
                'Unable to communicate with Mullvad kernel driver. Try reconnecting or send a problem report.',
              );
          }
      }
    }
  }

  private getTunnelParameterMessage(error: TunnelParameterError): string {
    const ipVersion = messages.pgettext('wireguard-settings-view', 'IP version');
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
      case TunnelParameterError.ipv4Unavailable:
        return sprintf(
          // TRANSLATORS: Label for notification when IPv4 is not available.
          messages.pgettext(
            'notifications',
            'IPv4 is not available, please try changing <b>%(ipVersionFeatureName)</b> setting.',
          ),
          { ipVersionFeatureName: ipVersion },
        );
      case TunnelParameterError.ipv6Unavailable:
        return sprintf(
          // TRANSLATORS: Label for notification when IPv6 is not available.
          messages.pgettext(
            'notifications',
            'IPv6 is not available, please try changing <b>%(ipVersionFeatureName)</b> setting.',
          ),
          { ipVersionFeatureName: ipVersion },
        );
    }
  }

  private getActions(errorState: ErrorStateDetails): InAppNotificationAction | void {
    const platform = process.platform ?? window.env.platform;

    if (errorState.cause === ErrorStateCause.setFirewallPolicyError && platform === 'linux') {
      return {
        type: 'troubleshoot-dialog',
        troubleshoot: {
          details: messages.pgettext('troubleshoot', 'This might be caused by an outdated kernel.'),
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
    } else if (errorState.cause === ErrorStateCause.needFullDiskPermissions) {
      let troubleshootButtons: InAppNotificationTroubleshootButton[] | undefined = undefined;
      if (this.context.showFullDiskAccessSettings) {
        troubleshootButtons = [
          {
            label: messages.pgettext('troubleshoot', 'Open system settings'),
            action: () => this.context.showFullDiskAccessSettings?.(),
            variant: 'success',
          },
          {
            label: messages.pgettext('troubleshoot', 'Disable split tunneling'),
            action: () => this.context.disableSplitTunneling?.(),
            variant: 'destructive',
          },
        ];
      }

      return {
        type: 'troubleshoot-dialog',
        troubleshoot: {
          details: messages.pgettext(
            'troubleshoot',
            'Failed to enable split tunneling. This is because the app is missing system permissions. What you can do:',
          ),
          steps: [
            messages.pgettext(
              'troubleshoot',
              'Enable “Full Disk Access” for “Mullvad VPN” in the macOS system settings.',
            ),
          ],
          buttons: troubleshootButtons,
        },
      };
    } else if (platform === 'win32' && errorState.cause === ErrorStateCause.splitTunnelError) {
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
    } else if (
      errorState.cause === ErrorStateCause.createTunnelDeviceError &&
      errorState.osError === 4319
    ) {
      return {
        type: 'troubleshoot-dialog',
        troubleshoot: {
          details: messages.pgettext(
            'troubleshoot',
            'Unable to start tunnel connection because of a failure when creating the tunnel device. This is often caused by conflicts with the VMware Bridge Protocol.',
          ),
          steps: [
            messages.pgettext('troubleshoot', 'Try to reinstall VMware.'),
            messages.pgettext('troubleshoot', 'Try to uninstall VMware.'),
          ],
        },
      };
    }
  }
}
