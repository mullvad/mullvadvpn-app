import moment from 'moment';
import * as React from 'react';
import { Component, Types } from 'reactxp';
import { sprintf } from 'sprintf-js';
import { messages } from '../../shared/gettext';
import {
  NotificationActions,
  NotificationBanner,
  NotificationContent,
  NotificationIndicator,
  NotificationOpenLinkAction,
  NotificationSubtitle,
  NotificationTitle,
} from './NotificationBanner';

import AccountExpiry from '../../shared/account-expiry';
import { ErrorStateCause, TunnelParameterError, TunnelState } from '../../shared/daemon-rpc-types';
import { parseAuthFailure } from '../lib/auth-failure';
import { IVersionReduxState } from '../redux/version/reducers';

interface IProps {
  style?: Types.ViewStyleRuleSet;
  accountExpiry?: AccountExpiry;
  tunnelState: TunnelState;
  version: IVersionReduxState;
  blockWhenDisconnected: boolean;
  onOpenDownloadLink: () => Promise<void>;
  onOpenBuyMoreLink: () => Promise<void>;
}

type NotificationAreaPresentation =
  | { type: 'failure-unsecured'; reason: string }
  | { type: 'blocking'; reason: string }
  | { type: 'inconsistent-version' }
  | { type: 'unsupported-version'; upgradeVersion: string }
  | { type: 'update-available'; upgradeVersion: string }
  | { type: 'expires-soon'; timeLeft: string };

type State = NotificationAreaPresentation & {
  visible: boolean;
};

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
        'WireGuard key not published to our servers. You can manage your key in Advanced settings.',
      );
    case 'custom_tunnel_host_resultion_error':
      return messages.pgettext(
        'in-app-notifications',
        'Failed to resolve host of custom tunnel. Consider changing the settings',
      );
  }
}

function getErrorCauseMessage(blockReason: ErrorStateCause): string {
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
}

function capitalizeFirstLetter(inputString: string): string {
  return inputString.charAt(0).toUpperCase() + inputString.slice(1);
}

export default class NotificationArea extends Component<IProps, State> {
  public static getDerivedStateFromProps(props: IProps, state: State) {
    const { accountExpiry, blockWhenDisconnected, tunnelState, version } = props;

    switch (tunnelState.state) {
      case 'connecting':
        return {
          visible: true,
          type: 'blocking',
          reason: '',
        };

      case 'error':
        if (tunnelState.details.isBlocking) {
          return {
            visible: true,
            type: 'blocking',
            reason: getErrorCauseMessage(tunnelState.details.cause),
          };
        } else {
          return {
            visible: true,
            type: 'failure-unsecured',
            reason: getErrorCauseMessage(tunnelState.details.cause),
          };
        }

      case 'disconnecting':
        if (tunnelState.details === 'reconnect') {
          return {
            visible: true,
            type: 'blocking',
            reason: '',
          };
        }
      // fallthrough

      case 'disconnected':
        if (blockWhenDisconnected) {
          return {
            visible: true,
            type: 'blocking',
            reason: '',
          };
        }
      // fallthrough

      default:
        if (!version.consistent) {
          return {
            visible: true,
            type: 'inconsistent-version',
          };
        }

        if (!version.currentIsSupported && version.nextUpgrade) {
          return {
            visible: true,
            type: 'unsupported-version',
            upgradeVersion: version.nextUpgrade,
          };
        }

        if (version.nextUpgrade && version.nextUpgrade !== version.current) {
          return {
            visible: true,
            type: 'update-available',
            upgradeVersion: version.nextUpgrade,
          };
        }

        if (accountExpiry && accountExpiry.willHaveExpiredAt(moment().add(3, 'days').toDate())) {
          return {
            visible: true,
            type: 'expires-soon',
            timeLeft: capitalizeFirstLetter(accountExpiry.remainingTime()),
          };
        }

        return {
          ...state,
          visible: false,
        };
    }
  }

  public state: State = {
    type: 'blocking',
    reason: '',
    visible: false,
  };

  public render() {
    return (
      <NotificationBanner style={this.props.style} visible={this.state.visible}>
        {this.state.type === 'failure-unsecured' && (
          <React.Fragment>
            <NotificationIndicator type={'error'} />
            <NotificationContent>
              <NotificationTitle>
                {messages.pgettext('in-app-notifications', 'YOU MIGHT BE LEAKING NETWORK TRAFFIC')}
              </NotificationTitle>
              <NotificationSubtitle>
                {messages.pgettext(
                  'in-app-notifications',
                  'Failed to block all network traffic. Please troubleshoot or report the problem to us.',
                )}
              </NotificationSubtitle>
            </NotificationContent>
          </React.Fragment>
        )}

        {this.state.type === 'blocking' && (
          <React.Fragment>
            <NotificationIndicator type={'error'} />
            <NotificationContent>
              <NotificationTitle>
                {messages.pgettext('in-app-notifications', 'BLOCKING INTERNET')}
              </NotificationTitle>
              <NotificationSubtitle>{this.state.reason}</NotificationSubtitle>
            </NotificationContent>
          </React.Fragment>
        )}

        {this.state.type === 'inconsistent-version' && (
          <React.Fragment>
            <NotificationIndicator type={'error'} />
            <NotificationContent>
              <NotificationTitle>
                {messages.pgettext('in-app-notifications', 'INCONSISTENT VERSION')}
              </NotificationTitle>
              <NotificationSubtitle>
                {messages.pgettext(
                  'in-app-notifications',
                  'Inconsistent internal version information, please restart the app',
                )}
              </NotificationSubtitle>
            </NotificationContent>
          </React.Fragment>
        )}

        {this.state.type === 'unsupported-version' && (
          <React.Fragment>
            <NotificationIndicator type={'error'} />
            <NotificationContent>
              <NotificationTitle>
                {messages.pgettext('in-app-notifications', 'UNSUPPORTED VERSION')}
              </NotificationTitle>
              <NotificationSubtitle>
                {sprintf(
                  // TRANSLATORS: The in-app banner displayed to the user when the running app becomes unsupported.
                  // TRANSLATORS: Available placeholders:
                  // TRANSLATORS: %(version)s - the newest available version of the app
                  messages.pgettext(
                    'in-app-notifications',
                    'You are running an unsupported app version. Please upgrade to %(version)s now to ensure your security',
                  ),
                  { version: this.state.upgradeVersion },
                )}
              </NotificationSubtitle>
            </NotificationContent>
            <NotificationActions>
              <NotificationOpenLinkAction onPress={this.props.onOpenDownloadLink} />
            </NotificationActions>
          </React.Fragment>
        )}

        {this.state.type === 'update-available' && (
          <React.Fragment>
            <NotificationIndicator type={'warning'} />
            <NotificationContent>
              <NotificationTitle>
                {messages.pgettext('in-app-notifications', 'UPDATE AVAILABLE')}
              </NotificationTitle>
              <NotificationSubtitle>
                {sprintf(
                  // TRANSLATORS: The in-app banner displayed to the user when the app update is available.
                  // TRANSLATORS: Available placeholders:
                  // TRANSLATORS: %(version)s - the newest available version of the app
                  messages.pgettext(
                    'in-app-notifications',
                    'Install Mullvad VPN (%(version)s) to stay up to date',
                  ),
                  { version: this.state.upgradeVersion },
                )}
              </NotificationSubtitle>
            </NotificationContent>
            <NotificationActions>
              <NotificationOpenLinkAction onPress={this.props.onOpenDownloadLink} />
            </NotificationActions>
          </React.Fragment>
        )}

        {this.state.type === 'expires-soon' && (
          <React.Fragment>
            <NotificationIndicator type={'warning'} />
            <NotificationContent>
              <NotificationTitle>
                {messages.pgettext('in-app-notifications', 'ACCOUNT CREDIT EXPIRES SOON')}
              </NotificationTitle>
              <NotificationSubtitle>{this.state.timeLeft}</NotificationSubtitle>
            </NotificationContent>
            <NotificationActions>
              <NotificationOpenLinkAction onPress={this.props.onOpenBuyMoreLink} />
            </NotificationActions>
          </React.Fragment>
        )}
      </NotificationBanner>
    );
  }
}
