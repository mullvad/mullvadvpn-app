import { useCallback, useState } from 'react';
import { useSelector } from 'react-redux';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import log from '../../shared/logging';
import { NewDeviceNotificationProvider } from '../../shared/notifications/new-device';
import {
  BlockWhenDisconnectedNotificationProvider,
  CloseToAccountExpiryNotificationProvider,
  ConnectingNotificationProvider,
  ErrorNotificationProvider,
  InAppNotificationAction,
  InAppNotificationProvider,
  InAppNotificationTroubleshootInfo,
  InconsistentVersionNotificationProvider,
  ReconnectingNotificationProvider,
  UnsupportedVersionNotificationProvider,
  UpdateAvailableNotificationProvider,
} from '../../shared/notifications/notification';
import { useAppContext } from '../context';
import useActions from '../lib/actionsHook';
import { transitions, useHistory } from '../lib/history';
import { formatHtml } from '../lib/html-formatter';
import { RoutePath } from '../lib/routes';
import accountActions from '../redux/account/actions';
import { IReduxState } from '../redux/store';
import * as AppButton from './AppButton';
import { ModalAlert, ModalAlertType, ModalMessage } from './Modal';
import {
  NotificationActions,
  NotificationBanner,
  NotificationCloseAction,
  NotificationContent,
  NotificationIndicator,
  NotificationOpenLinkAction,
  NotificationSubtitle,
  NotificationTitle,
  NotificationTroubleshootDialogAction,
} from './NotificationBanner';

interface IProps {
  className?: string;
}

export default function NotificationArea(props: IProps) {
  const account = useSelector((state: IReduxState) => state.account);
  const locale = useSelector((state: IReduxState) => state.userInterface.locale);
  const tunnelState = useSelector((state: IReduxState) => state.connection.status);
  const version = useSelector((state: IReduxState) => state.version);
  const blockWhenDisconnected = useSelector(
    (state: IReduxState) => state.settings.blockWhenDisconnected,
  );
  const hasExcludedApps = useSelector(
    (state: IReduxState) =>
      state.settings.splitTunneling && state.settings.splitTunnelingApplications.length > 0,
  );

  const { hideNewDeviceBanner } = useActions(accountActions);

  const notificationProviders: InAppNotificationProvider[] = [
    new ConnectingNotificationProvider({ tunnelState }),
    new ReconnectingNotificationProvider(tunnelState),
    new BlockWhenDisconnectedNotificationProvider({
      tunnelState,
      blockWhenDisconnected,
      hasExcludedApps,
    }),
    new ErrorNotificationProvider({ tunnelState, hasExcludedApps }),
    new InconsistentVersionNotificationProvider({ consistent: version.consistent }),
    new UnsupportedVersionNotificationProvider(version),
  ];

  if (account.expiry) {
    notificationProviders.push(
      new CloseToAccountExpiryNotificationProvider({ accountExpiry: account.expiry, locale }),
    );
  }

  notificationProviders.push(
    new NewDeviceNotificationProvider({
      shouldDisplay: account.status.type === 'ok' && account.status.newDeviceBanner,
      deviceName: account.deviceName ?? '',
      close: hideNewDeviceBanner,
    }),
    new UpdateAvailableNotificationProvider(version),
  );

  const notificationProvider = notificationProviders.find((notification) =>
    notification.mayDisplay(),
  );

  if (notificationProvider) {
    const notification = notificationProvider.getInAppNotification();

    if (notification) {
      return (
        <NotificationBanner className={props.className}>
          <NotificationIndicator $type={notification.indicator} />
          <NotificationContent role="status" aria-live="polite">
            <NotificationTitle>{notification.title}</NotificationTitle>
            <NotificationSubtitle>{formatHtml(notification.subtitle ?? '')}</NotificationSubtitle>
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

  return <NotificationBanner className={props.className} aria-hidden={true} />;
}

const TroubleshootList = styled.ul({
  listStyle: 'disc outside',
  paddingLeft: '20px',
  color: colors.white80,
});

interface INotificationActionWrapperProps {
  action: InAppNotificationAction;
}

function NotificationActionWrapper(props: INotificationActionWrapperProps) {
  const history = useHistory();
  const { openLinkWithAuth, openUrl } = useAppContext();
  const [troubleshootInfo, setTroubleshootInfo] = useState<InAppNotificationTroubleshootInfo>();

  const handleClick = useCallback(() => {
    if (props.action) {
      switch (props.action.type) {
        case 'open-url':
          if (props.action.withAuth) {
            return openLinkWithAuth(props.action.url);
          } else {
            return openUrl(props.action.url);
          }
        case 'troubleshoot-dialog':
          setTroubleshootInfo(props.action.troubleshoot);
          break;
        case 'close':
          props.action.close();
          break;
      }
    }

    return Promise.resolve();
  }, [props.action]);

  const goToProblemReport = useCallback(() => {
    setTroubleshootInfo(undefined);
    history.push(RoutePath.problemReport, { transition: transitions.show });
  }, []);

  const closeTroubleshootInfo = useCallback(() => setTroubleshootInfo(undefined), []);

  let actionComponent: React.ReactElement | undefined;
  if (props.action) {
    switch (props.action.type) {
      case 'open-url':
        actionComponent = <NotificationOpenLinkAction onClick={handleClick} />;
        break;
      case 'troubleshoot-dialog':
        actionComponent = (
          <>
            <NotificationTroubleshootDialogAction onClick={handleClick} />
          </>
        );
        break;
      case 'close':
        actionComponent = <NotificationCloseAction onClick={handleClick} />;
    }
  }

  return (
    <>
      <NotificationActions>{actionComponent}</NotificationActions>
      <ModalAlert
        isOpen={troubleshootInfo !== undefined}
        type={ModalAlertType.info}
        buttons={[
          <AppButton.GreenButton key="problem-report" onClick={goToProblemReport}>
            {messages.pgettext('in-app-notifications', 'Send problem report')}
          </AppButton.GreenButton>,
          <AppButton.BlueButton key="back" onClick={closeTroubleshootInfo}>
            {messages.gettext('Back')}
          </AppButton.BlueButton>,
        ]}
        close={closeTroubleshootInfo}>
        <ModalMessage>{troubleshootInfo?.details}</ModalMessage>
        <ModalMessage>
          <TroubleshootList>
            {troubleshootInfo?.steps.map((step) => (
              <li key={step}>{step}</li>
            ))}
          </TroubleshootList>
        </ModalMessage>
        <ModalMessage>
          {messages.pgettext(
            'troubleshoot',
            'If these steps do not work please send a problem report.',
          )}
        </ModalMessage>
      </ModalAlert>
    </>
  );
}
