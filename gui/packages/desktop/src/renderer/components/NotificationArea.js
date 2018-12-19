// @flow

import moment from 'moment';
import * as React from 'react';
import { Component, Types } from 'reactxp';
import {
  NotificationBanner,
  NotificationIndicator,
  NotificationContent,
  NotificationActions,
  NotificationTitle,
  NotificationSubtitle,
  NotificationOpenLinkAction,
} from './NotificationBanner';

import { AuthFailure } from '../lib/auth-failure';
import AccountExpiry from '../lib/account-expiry';
import type { BlockReason, TunnelStateTransition } from '../lib/daemon-rpc-proxy';
import type { VersionReduxState } from '../redux/version/reducers';

type Props = {
  style?: Types.ViewStyleRuleSet,
  accountExpiry: ?AccountExpiry,
  tunnelState: TunnelStateTransition,
  version: VersionReduxState,
  openExternalLink: (string) => void,
  blockWhenDisconnected: boolean,
};

type NotificationAreaPresentation =
  | { type: 'blocking', reason: string }
  | { type: 'inconsistent-version' }
  | { type: 'unsupported-version', upgradeVersion: string }
  | { type: 'update-available', upgradeVersion: string }
  | { type: 'expires-soon', timeLeft: string };

type State = NotificationAreaPresentation & {
  visible: boolean,
};

function getBlockReasonMessage(blockReason: BlockReason): string {
  switch (blockReason.reason) {
    case 'auth_failed': {
      return new AuthFailure(blockReason.details).show();
    }
    case 'ipv6_unavailable':
      return 'Could not configure IPv6, please enable it on your system or disable it in the app';
    case 'set_security_policy_error':
      return 'Failed to apply security policy';
    case 'set_dns_error':
      return 'Failed to set system DNS server';
    case 'start_tunnel_error':
      return 'Failed to start tunnel connection';
    case 'no_matching_relay':
      return 'No relay server matches the current settings';
    case 'is_offline':
      return 'This device is offline, no tunnels can be established';
    case 'tap_adapter_problem':
      return "Unable to detect a working TAP adapter on this device. If you've disabled it, enable it again. Otherwise, please reinstall the app";
    default:
      return `Unknown error: ${(blockReason.reason: empty)}`;
  }
}

export default class NotificationArea extends Component<Props, State> {
  state = {
    type: 'blocking',
    reason: '',
    visible: false,
  };

  static getDerivedStateFromProps(props: Props, state: State) {
    const { accountExpiry, blockWhenDisconnected, tunnelState, version } = props;

    switch (tunnelState.state) {
      case 'connecting':
        return {
          visible: true,
          type: 'blocking',
          reason: '',
        };

      case 'blocked':
        return {
          visible: true,
          type: 'blocking',
          reason: getBlockReasonMessage(tunnelState.details),
        };

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
            timeLeft: NotificationArea._capitalizeFirstLetter(accountExpiry.remainingTime()),
          };
        }

        return {
          ...state,
          visible: false,
        };
    }
  }

  static _capitalizeFirstLetter(initialString: string): string {
    return initialString.length > 0
      ? initialString.charAt(0).toUpperCase() + initialString.slice(1)
      : '';
  }

  render() {
    return (
      <NotificationBanner style={this.props.style} visible={this.state.visible}>
        {this.state.type === 'blocking' && (
          <React.Fragment>
            <NotificationIndicator type={'error'} />
            <NotificationContent>
              <NotificationTitle>{'BLOCKING INTERNET'}</NotificationTitle>
              <NotificationSubtitle>{this.state.reason}</NotificationSubtitle>
            </NotificationContent>
          </React.Fragment>
        )}

        {this.state.type === 'inconsistent-version' && (
          <React.Fragment>
            <NotificationIndicator type={'error'} />
            <NotificationContent>
              <NotificationTitle>{'INCONSISTENT VERSION'}</NotificationTitle>
              <NotificationSubtitle>
                {'Inconsistent internal version information, please restart the app'}
              </NotificationSubtitle>
            </NotificationContent>
          </React.Fragment>
        )}

        {this.state.type === 'unsupported-version' && (
          <React.Fragment>
            <NotificationIndicator type={'error'} />
            <NotificationContent>
              <NotificationTitle>{'UNSUPPORTED VERSION'}</NotificationTitle>
              <NotificationSubtitle>{`You are running an unsupported app version. Please upgrade to ${
                this.state.upgradeVersion
              } now to ensure your security`}</NotificationSubtitle>
            </NotificationContent>
            <NotificationActions>
              <NotificationOpenLinkAction
                onPress={() => {
                  this.props.openExternalLink('download');
                }}
              />
            </NotificationActions>
          </React.Fragment>
        )}

        {this.state.type === 'update-available' && (
          <React.Fragment>
            <NotificationIndicator type={'warning'} />
            <NotificationContent>
              <NotificationTitle>{`UPDATE AVAILABLE`}</NotificationTitle>
              <NotificationSubtitle>{`Install Mullvad VPN (${
                this.state.upgradeVersion
              }) to stay up to date`}</NotificationSubtitle>
            </NotificationContent>
            <NotificationActions>
              <NotificationOpenLinkAction
                onPress={() => {
                  this.props.openExternalLink('download');
                }}
              />
            </NotificationActions>
          </React.Fragment>
        )}

        {this.state.type === 'expires-soon' && (
          <React.Fragment>
            <NotificationIndicator type={'warning'} />
            <NotificationContent>
              <NotificationTitle>{'ACCOUNT CREDIT EXPIRES SOON'}</NotificationTitle>
              <NotificationSubtitle>{this.state.timeLeft}</NotificationSubtitle>
            </NotificationContent>
            <NotificationActions>
              <NotificationOpenLinkAction
                onPress={() => {
                  this.props.openExternalLink('purchase');
                }}
              />
            </NotificationActions>
          </React.Fragment>
        )}
      </NotificationBanner>
    );
  }
}
