import { useCallback, useState } from 'react';

import { messages } from '../../shared/gettext';
import log from '../../shared/logging';
import {
  BlockWhenDisconnectedNotificationProvider,
  CloseToAccountExpiryNotificationProvider,
  ConnectingNotificationProvider,
  ErrorNotificationProvider,
  InAppNotificationAction,
  InAppNotificationProvider,
  InconsistentVersionNotificationProvider,
  ReconnectingNotificationProvider,
  UnsupportedVersionNotificationProvider,
  UpdateAvailableNotificationProvider,
} from '../../shared/notifications';
import { useAppContext } from '../context';
import useActions from '../lib/actionsHook';
import { Link } from '../lib/components';
import { Colors } from '../lib/foundations';
import { transitions, useHistory } from '../lib/history';
import { formatHtml } from '../lib/html-formatter';
import {
  NewDeviceNotificationProvider,
  NewVersionNotificationProvider,
} from '../lib/notifications';
import { RoutePath } from '../lib/routes';
import accountActions from '../redux/account/actions';
import { IReduxState, useSelector } from '../redux/store';
import * as AppButton from './AppButton';
import { ModalAlert, ModalAlertType, ModalMessage, ModalMessageList } from './Modal';
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
  const { showFullDiskAccessSettings } = useAppContext();

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

  const { setDisplayedChangelog, setDismissedUpgrade } = useAppContext();

  const currentVersion = useSelector((state) => state.version.current);
  const displayedForVersion = useSelector(
    (state) => state.settings.guiSettings.changelogDisplayedForVersion,
  );
  const updateDismissedForVersion = useSelector(
    (state) => state.settings.guiSettings.updateDismissedForVersion,
  );
  const changelog = useSelector((state) => state.userInterface.changelog);

  const displayedChangelog = useCallback(() => {
    setDisplayedChangelog();
  }, [setDisplayedChangelog]);

  const [isModalOpen, setIsModalOpen] = useState(false);

  const { setSplitTunnelingState } = useAppContext();
  const disableSplitTunneling = useCallback(async () => {
    setIsModalOpen(false);
    await setSplitTunnelingState(false);
  }, [setSplitTunnelingState]);

  const notificationProviders: InAppNotificationProvider[] = [
    new ConnectingNotificationProvider({ tunnelState }),
    new ReconnectingNotificationProvider(tunnelState),
    new BlockWhenDisconnectedNotificationProvider({
      tunnelState,
      blockWhenDisconnected,
      hasExcludedApps,
    }),

    new ErrorNotificationProvider({
      tunnelState,
      hasExcludedApps,
      showFullDiskAccessSettings,
      disableSplitTunneling,
    }),
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
    new NewVersionNotificationProvider({
      currentVersion,
      displayedForVersion,
      changelog,
      close: displayedChangelog,
    }),
    new UpdateAvailableNotificationProvider({
      suggestedIsBeta: version.suggestedIsBeta,
      suggestedUpgrade: version.suggestedUpgrade,
      close: setDismissedUpgrade,
      updateDismissedForVersion,
    }),
  );

  const notificationProvider = notificationProviders.find((notification) =>
    notification.mayDisplay(),
  );

  if (notificationProvider) {
    const notification = notificationProvider.getInAppNotification();

    if (notification) {
      return (
        <NotificationBanner className={props.className} data-testid="notificationBanner">
          <NotificationIndicator
            $type={notification.indicator}
            data-testid="notificationIndicator"
          />
          <NotificationContent role="status" aria-live="polite">
            <NotificationTitle data-testid="notificationTitle">
              {notification.title}
            </NotificationTitle>
            <NotificationSubtitle data-testid="notificationSubTitle">
              {Array.isArray(notification.subtitle)
                ? notification.subtitle.map((subtitle, index, arr) => {
                    const content =
                      subtitle.action && subtitle.action.type === 'navigate' ? (
                        <Link variant="labelTiny" color={Colors.white60} {...subtitle.action.link}>
                          {formatHtml(subtitle.content)}
                        </Link>
                      ) : (
                        formatHtml(subtitle.content)
                      );
                    return (
                      <span key={index}>
                        {content}
                        {index !== arr.length - 1 && ' '}
                      </span>
                    );
                  })
                : formatHtml(notification.subtitle ?? '')}
            </NotificationSubtitle>
          </NotificationContent>
          {notification.action && (
            <NotificationActionWrapper
              action={notification.action}
              isModalOpen={isModalOpen}
              setIsModalOpen={setIsModalOpen}
            />
          )}
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

interface NotificationActionWrapperProps {
  action: InAppNotificationAction;
  isModalOpen: boolean;
  setIsModalOpen: (isOpen: boolean) => void;
}

function NotificationActionWrapper({
  action,
  isModalOpen,
  setIsModalOpen,
}: NotificationActionWrapperProps) {
  const { push } = useHistory();
  const { openUrlWithAuth, openUrl } = useAppContext();

  const closeModal = useCallback(() => setIsModalOpen(false), [setIsModalOpen]);

  const handleClick = useCallback(() => {
    if (action) {
      switch (action.type) {
        case 'open-url':
          if (action.withAuth) {
            return openUrlWithAuth(action.url);
          } else {
            return openUrl(action.url);
          }
        case 'troubleshoot-dialog':
          setIsModalOpen(true);
          break;
        case 'close':
          if (action.close) {
            action.close();
          }
          break;
      }
    }

    return Promise.resolve();
  }, [action, setIsModalOpen, openUrlWithAuth, openUrl]);

  const goToProblemReport = useCallback(() => {
    closeModal();
    push(RoutePath.problemReport, { transition: transitions.show });
  }, [closeModal, push]);

  let actionComponent: React.ReactElement | undefined;
  if (action) {
    switch (action.type) {
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

  if (action.type !== 'troubleshoot-dialog') {
    return <NotificationActions>{actionComponent}</NotificationActions>;
  }

  const problemReportButton = action.troubleshoot?.buttons ? (
    <AppButton.BlueButton key="problem-report" onClick={goToProblemReport}>
      {messages.pgettext('in-app-notifications', 'Send problem report')}
    </AppButton.BlueButton>
  ) : (
    <AppButton.GreenButton key="problem-report" onClick={goToProblemReport}>
      {messages.pgettext('in-app-notifications', 'Send problem report')}
    </AppButton.GreenButton>
  );

  let buttons = [
    problemReportButton,
    <AppButton.BlueButton key="back" onClick={closeModal}>
      {messages.gettext('Back')}
    </AppButton.BlueButton>,
  ];

  if (action.troubleshoot?.buttons) {
    const actionButtons = action.troubleshoot.buttons.map(({ variant, label, action }) => {
      if (variant === 'success')
        return (
          <AppButton.GreenButton key={label} onClick={action}>
            {label}
          </AppButton.GreenButton>
        );
      else if (variant === 'destructive')
        return (
          <AppButton.RedButton key={label} onClick={action}>
            {label}
          </AppButton.RedButton>
        );
      else
        return (
          <AppButton.BlueButton key={label} onClick={action}>
            {label}
          </AppButton.BlueButton>
        );
    });

    buttons = actionButtons.concat(buttons);
  }

  return (
    <>
      <NotificationActions>{actionComponent}</NotificationActions>
      <ModalAlert
        isOpen={isModalOpen}
        type={ModalAlertType.info}
        buttons={buttons}
        close={closeModal}>
        <ModalMessage>{action.troubleshoot?.details}</ModalMessage>
        <ModalMessage>
          <ModalMessageList>
            {action.troubleshoot?.steps.map((step) => <li key={step}>{step}</li>)}
          </ModalMessageList>
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
