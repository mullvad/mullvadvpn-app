import React, { useCallback } from 'react';
import { useSelector } from 'react-redux';
import log from '../../shared/logging';
import {
  BlockWhenDisconnectedNotificationProvider,
  CloseToAccountExpiryNotificationProvider,
  ErrorNotificationProvider,
  InAppNotificationProvider,
  InconsistentVersionNotificationProvider,
  NotificationAction,
  NoValidKeyNotificationProvider,
  UnsupportedVersionNotificationProvider,
  UpdateAvailableNotificationProvider,
} from '../../shared/notifications/notification';
import { useAppContext } from '../context';
import { IReduxState } from '../redux/store';
import {
  NotificationActions,
  NotificationBanner,
  NotificationContent,
  NotificationIndicator,
  NotificationOpenLinkAction,
  NotificationSubtitle,
  NotificationTitle,
} from './NotificationBanner';

interface IProps {
  className?: string;
}

export default function NotificationArea(props: IProps) {
  const accountExpiry = useSelector((state: IReduxState) => state.account.expiry);
  const locale = useSelector((state: IReduxState) => state.userInterface.locale);
  const tunnelState = useSelector((state: IReduxState) => state.connection.status);
  const version = useSelector((state: IReduxState) => state.version);
  const blockWhenDisconnected = useSelector(
    (state: IReduxState) => state.settings.blockWhenDisconnected,
  );
  const tunnelProtocol = useSelector((state: IReduxState) =>
    'normal' in state.settings.relaySettings
      ? state.settings.relaySettings.normal.tunnelProtocol
      : undefined,
  );
  const wireGuardKey = useSelector((state: IReduxState) => state.settings.wireguardKeyState);

  const notificationProviders: InAppNotificationProvider[] = [
    new BlockWhenDisconnectedNotificationProvider({ tunnelState, blockWhenDisconnected }),
    new ErrorNotificationProvider({ tunnelState, accountExpiry }),
    new NoValidKeyNotificationProvider({ tunnelProtocol, wireGuardKey }),
    new InconsistentVersionNotificationProvider({ consistent: version.consistent }),
    new UnsupportedVersionNotificationProvider(version),
  ];

  if (accountExpiry) {
    notificationProviders.push(
      new CloseToAccountExpiryNotificationProvider({ accountExpiry, locale }),
    );
  }

  notificationProviders.push(new UpdateAvailableNotificationProvider(version));

  const notificationProvider = notificationProviders.find((notification) =>
    notification.mayDisplay(),
  );

  if (notificationProvider) {
    const notification = notificationProvider.getInAppNotification();

    if (notification) {
      return (
        <NotificationBanner className={props.className} visible>
          <NotificationIndicator type={notification.indicator} />
          <NotificationContent role="status" aria-live="polite">
            <NotificationTitle>{notification.title}</NotificationTitle>
            <NotificationSubtitle>{notification.subtitle}</NotificationSubtitle>
          </NotificationContent>
          {notification.action && <NotificationActionWrapper action={notification.action} />}
        </NotificationBanner>
      );
    } else {
      log.error(
        `Notification providers mayDisplay() returned true but getInAppNotification() returned undefined for ${notificationProvider.constructor.name}`,
      );
    }
  }

  return <NotificationBanner className={props.className} visible={false} aria-hidden={true} />;
}

interface INotificationActionWrapperProps {
  action: NotificationAction;
}

function NotificationActionWrapper(props: INotificationActionWrapperProps) {
  const { openLinkWithAuth, openUrl } = useAppContext();

  const handleClick = useCallback(() => {
    if (props.action.withAuth) {
      return openLinkWithAuth(props.action.url);
    } else {
      return openUrl(props.action.url);
    }
  }, []);

  return (
    <NotificationActions>
      <NotificationOpenLinkAction onClick={handleClick} />
    </NotificationActions>
  );
}
