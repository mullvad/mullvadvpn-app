import moment from 'moment';
import * as React from 'react';
import { Component, Types } from 'reactxp';
import { links } from '../../config.json';
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
      return 'Could not configure IPv6, please enable it on your system or disable it in the app';
    case 'set_firewall_policy_error':
      return 'Failed to apply firewall rules. The device might currently be unsecured';
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
              <NotificationTitle>{'FAILURE - UNSECURED'}</NotificationTitle>
              <NotificationSubtitle>{this.state.reason}</NotificationSubtitle>
            </NotificationContent>
          </React.Fragment>
        )}

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
              <NotificationOpenLinkAction onPress={this.handleOpenDownloadLink} />
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
              <NotificationOpenLinkAction onPress={this.handleOpenDownloadLink} />
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
