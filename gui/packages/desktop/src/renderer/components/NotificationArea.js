// @flow
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

import type { BlockReason, TunnelStateTransition } from '../lib/daemon-rpc';
import type { VersionReduxState } from '../redux/version/reducers';

type Props = {
  style?: Types.ViewStyleRuleSet,
  tunnelState: TunnelStateTransition,
  version: VersionReduxState,
  openExternalLink: (string) => void,
};

type NotificationAreaPresentation =
  | { type: 'blocking', reason: string }
  | { type: 'inconsistent-version' }
  | { type: 'unsupported-version', upgradeVersion: string }
  | { type: 'update-available', upgradeVersion: string };

type State = NotificationAreaPresentation & {
  visible: boolean,
};

function getBlockReasonMessage(blockReason: BlockReason): string {
  switch (blockReason.reason) {
    case 'auth_failed': {
      const details =
        blockReason.details ||
        'Check that the account is valid, has time left and not too many connections';
      return `Authentication failed: ${details}`;
    }
    case 'ipv6_unavailable':
      return 'Could not configure IPv6, please enable it on your system or disable it in the app';
    case 'set_security_policy_error':
      return 'Failed to apply security policy';
    case 'start_tunnel_error':
      return 'Failed to start tunnel connection';
    case 'no_matching_relay':
      return 'No relay server matches the current settings';
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
    const { version, tunnelState } = props;

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

        return {
          ...state,
          visible: false,
        };
    }
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
              <NotificationSubtitle>{`This app version might have security issues. Please upgrade to ${
                this.state.upgradeVersion
              }`}</NotificationSubtitle>
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
      </NotificationBanner>
    );
  }
}
