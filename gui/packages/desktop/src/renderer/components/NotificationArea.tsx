import moment from 'moment';
import * as React from 'react';
import { Component, Types } from 'reactxp';
import { sprintf } from 'sprintf-js';
import { links } from '../../config.json';
import { pgettext } from '../../shared/gettext';
import {
  NotificationActions,
  NotificationBanner,
  NotificationContent,
  NotificationIndicator,
  NotificationOpenLinkAction,
  NotificationSubtitle,
  NotificationTitle,
} from './NotificationBanner';

import { BlockReason, TunnelStateTransition } from '../../shared/daemon-rpc-types';
import AccountExpiry from '../lib/account-expiry';
import { AuthFailure } from '../lib/auth-failure';
import { IVersionReduxState } from '../redux/version/reducers';

interface IProps {
  style?: Types.ViewStyleRuleSet;
  accountExpiry?: AccountExpiry;
  tunnelState: TunnelStateTransition;
  version: IVersionReduxState;
  openExternalLink: (url: string) => void;
  blockWhenDisconnected: boolean;
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

function getBlockReasonMessage(blockReason: BlockReason): string {
  switch (blockReason.reason) {
    case 'auth_failed': {
      return new AuthFailure(blockReason.details).show();
    }
    case 'ipv6_unavailable':
      // TRANSLATORS: Could not configure IPv6, please enable it on your system or disable it in the app
      return pgettext('in-app-notifications', 'ipv6-unavailable');
    case 'set_firewall_policy_error':
      // TRANSLATORS: Failed to apply firewall rules. The device might currently be unsecured
      return pgettext('in-app-notifications', 'set-firewall-policy-error');
    case 'set_dns_error':
      // TRANSLATORS: Failed to set system DNS server
      return pgettext('in-app-notifications', 'set-dns-error');
    case 'start_tunnel_error':
      // TRANSLATORS: Failed to start tunnel connection
      return pgettext('in-app-notifications', 'start-tunnel-error');
    case 'no_matching_relay':
      // TRANSLATORS: No relay server matches the current settings
      return pgettext('in-app-notifications', 'no-matching-relay');
    case 'is_offline':
      // TRANSLATORS: This device is offline, no tunnels can be established
      return pgettext('in-app-notifications', 'is-offline');
    case 'tap_adapter_problem':
      // TRANSLATORS: Unable to detect a working TAP adapter on this device. If you've disabled it, enable it again. Otherwise, please reinstall the app
      return pgettext('in-app-notifications', 'tap-adapter-problem');
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

      case 'blocked':
        switch (tunnelState.details.reason) {
          case 'set_firewall_policy_error':
            return {
              visible: true,
              type: 'failure-unsecured',
              reason: getBlockReasonMessage(tunnelState.details),
            };
          default:
            return {
              visible: true,
              type: 'blocking',
              reason: getBlockReasonMessage(tunnelState.details),
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

        if (!version.upToDate && version.nextUpgrade) {
          return {
            visible: true,
            type: 'update-available',
            upgradeVersion: version.nextUpgrade,
          };
        }

        if (accountExpiry && accountExpiry.willHaveExpiredIn(moment().add(3, 'days'))) {
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
                {// TRANSLATORS: FAILURE - UNSECURED
                pgettext('in-app-notifications', 'failure-unsecured-notification-title')}
              </NotificationTitle>
              <NotificationSubtitle>{this.state.reason}</NotificationSubtitle>
            </NotificationContent>
          </React.Fragment>
        )}

        {this.state.type === 'blocking' && (
          <React.Fragment>
            <NotificationIndicator type={'error'} />
            <NotificationContent>
              <NotificationTitle>
                {// TRANSLATORS: BLOCKING INTERNET
                pgettext('in-app-notifications', 'blocking-notification-title')}
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
                {// TRANSLATORS: INCONSISTENT VERSION
                pgettext('in-app-notifications', 'inconsistent-version-notification-title')}
              </NotificationTitle>
              <NotificationSubtitle>
                {// TRANSLATORS: Inconsistent internal version information, please restart the app
                pgettext('in-app-notifications', 'inconsistent-version-notification-subtitle')}
              </NotificationSubtitle>
            </NotificationContent>
          </React.Fragment>
        )}

        {this.state.type === 'unsupported-version' && (
          <React.Fragment>
            <NotificationIndicator type={'error'} />
            <NotificationContent>
              <NotificationTitle>
                {// TRANSLATORS: UNSUPPORTED VERSION
                pgettext('in-app-notifications', 'unsupported-version-notification-title')}
              </NotificationTitle>
              <NotificationSubtitle>
                {sprintf(
                  // TRANSLATORS: You are running an unsupported app version. Please upgrade to %(version)s now to ensure your security
                  pgettext('in-app-notifications', 'unsupported-version-notification-subtitle'),
                  { version: this.state.upgradeVersion },
                )}
              </NotificationSubtitle>
            </NotificationContent>
            <NotificationActions>
              <NotificationOpenLinkAction onPress={this.handleOpenDownloadLink} />
            </NotificationActions>
          </React.Fragment>
        )}

        {this.state.type === 'update-available' && (
          <React.Fragment>
            <NotificationIndicator type={'warning'} />
            <NotificationContent>
              <NotificationTitle>
                {// TRANSLATORS: UPDATE AVAILABLE
                pgettext('in-app-notifications', 'update-available-notification-title')}
              </NotificationTitle>
              <NotificationSubtitle>
                {sprintf(
                  // TRANSLATORS: Install Mullvad VPN (%(version)s) to stay up to date
                  pgettext('in-app-notifications', 'update-available-notification-subtitle'),
                  { version: this.state.upgradeVersion },
                )}
              </NotificationSubtitle>
            </NotificationContent>
            <NotificationActions>
              <NotificationOpenLinkAction onPress={this.handleOpenDownloadLink} />
            </NotificationActions>
          </React.Fragment>
        )}

        {this.state.type === 'expires-soon' && (
          <React.Fragment>
            <NotificationIndicator type={'warning'} />
            <NotificationContent>
              <NotificationTitle>
                {// TRANSLATORS: ACCOUNT CREDIT EXPIRES SOON
                pgettext('in-app-notifications', 'expires-soon-notification-title')}
              </NotificationTitle>
              <NotificationSubtitle>{this.state.timeLeft}</NotificationSubtitle>
            </NotificationContent>
            <NotificationActions>
              <NotificationOpenLinkAction onPress={this.handleOpenBuyMoreLink} />
            </NotificationActions>
          </React.Fragment>
        )}
      </NotificationBanner>
    );
  }

  private handleOpenDownloadLink = () => {
    this.props.openExternalLink(links.download);
  };

  private handleOpenBuyMoreLink = () => {
    this.props.openExternalLink(links.purchase);
  };
}
