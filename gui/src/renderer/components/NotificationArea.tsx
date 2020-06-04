import * as React from 'react';
import { Component, Types } from 'reactxp';
import { sprintf } from 'sprintf-js';
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
import * as notifications from '../../shared/notifications/notification';
import { TunnelState } from '../../shared/daemon-rpc-types';
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

function capitalizeFirstLetter(inputString: string): string {
  return inputString.charAt(0).toUpperCase() + inputString.slice(1);
}

export default class NotificationArea extends Component<IProps> {
  public render() {
    if (notifications.connecting.condition(this.props.tunnelState)) {
      return this.renderNotification(notifications.connecting.inAppNotification);
    } else if (notifications.reconnecting.condition(this.props.tunnelState)) {
      return this.renderNotification(notifications.reconnecting.inAppNotification);
    } else if (
      notifications.blockWhenDisconnected.condition(
        this.props.tunnelState,
        this.props.blockWhenDisconnected,
      )
    ) {
      return this.renderNotification(notifications.blockWhenDisconnected.inAppNotification);
    } else if (notifications.nonBlockingError.condition(this.props.tunnelState)) {
      return this.renderNotification(notifications.nonBlockingError.inAppNotification);
    } else if (notifications.error.condition(this.props.tunnelState)) {
      const inAppNotification = notifications.error.inAppNotification;
      return (
        <NotificationBanner style={this.props.style} visible>
          <NotificationIndicator type={inAppNotification.indicator} />
          <NotificationContent>
            <NotificationTitle>{inAppNotification.title}</NotificationTitle>
            <NotificationSubtitle>
              {sprintf(inAppNotification.body(this.props.tunnelState), {
                version: this.props.version.nextUpgrade,
              })}
            </NotificationSubtitle>
          </NotificationContent>
          <NotificationActions>
            <NotificationOpenLinkAction onPress={this.props.onOpenDownloadLink} />
          </NotificationActions>
        </NotificationBanner>
      );
    } else if (notifications.inconsistentVersion.condition(this.props.version.consistent)) {
      return this.renderNotification(notifications.inconsistentVersion.inAppNotification);
    } else if (
      notifications.unsupportedVersion.condition(
        this.props.version.supported,
        this.props.version.consistent,
        this.props.version.nextUpgrade,
      )
    ) {
      const inAppNotification = notifications.unsupportedVersion.inAppNotification;
      return (
        <NotificationBanner style={this.props.style} visible>
          <NotificationIndicator type={inAppNotification.indicator} />
          <NotificationContent>
            <NotificationTitle>{inAppNotification.title}</NotificationTitle>
            <NotificationSubtitle>
              {sprintf(inAppNotification.body, {
                version: this.props.version.nextUpgrade,
              })}
            </NotificationSubtitle>
          </NotificationContent>
          <NotificationActions>
            <NotificationOpenLinkAction onPress={this.props.onOpenDownloadLink} />
          </NotificationActions>
        </NotificationBanner>
      );
    } else if (
      notifications.updateAvailable.condition(
        this.props.version.nextUpgrade,
        this.props.version.current,
      )
    ) {
      const inAppNotification = notifications.updateAvailable.inAppNotification;
      return (
        <NotificationBanner style={this.props.style} visible>
          <NotificationIndicator type={inAppNotification.indicator} />
          <NotificationContent>
            <NotificationTitle>{inAppNotification.title}</NotificationTitle>
            <NotificationSubtitle>
              {sprintf(inAppNotification.body, {
                version: this.props.version.nextUpgrade,
              })}
            </NotificationSubtitle>
          </NotificationContent>
          <NotificationActions>
            <NotificationOpenLinkAction onPress={this.props.onOpenDownloadLink} />
          </NotificationActions>
        </NotificationBanner>
      );
    } else if (notifications.accountExpiry.condition(this.props.accountExpiry)) {
      const inAppNotification = notifications.accountExpiry.inAppNotification;
      return (
        <NotificationBanner style={this.props.style} visible>
          <NotificationIndicator type={inAppNotification.indicator} />
          <NotificationContent>
            <NotificationTitle>{inAppNotification.title}</NotificationTitle>
            <NotificationSubtitle>
              {sprintf(inAppNotification.body, {
                duration: capitalizeFirstLetter(this.props.accountExpiry?.remainingTime() ?? ''),
              })}
            </NotificationSubtitle>
          </NotificationContent>
          <NotificationActions>
            <NotificationOpenLinkAction onPress={this.props.onOpenBuyMoreLink} />
          </NotificationActions>
        </NotificationBanner>
      );
    } else {
      return <NotificationBanner style={this.props.style} visible={false} />;
    }
  }

  private renderNotification(notification?: notifications.InAppNotification<never>) {
    if (notification === undefined) {
      return null;
    } else {
      return (
        <NotificationBanner style={this.props.style} visible>
          <NotificationIndicator type={notification.indicator} />
          <NotificationContent>
            <NotificationTitle>{notification.title}</NotificationTitle>
            {notification.body && <NotificationSubtitle>{notification.body}</NotificationSubtitle>}
          </NotificationContent>
        </NotificationBanner>
      );
    }
  }
}
