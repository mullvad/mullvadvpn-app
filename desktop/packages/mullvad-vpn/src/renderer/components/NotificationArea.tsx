import { AnimatePresence } from 'motion/react';
import { useCallback, useState } from 'react';

import { messages } from '../../shared/gettext';
import log from '../../shared/logging';
import {
  CloseToAccountExpiryNotificationProvider,
  ConnectingNotificationProvider,
  ErrorNotificationProvider,
  InAppNotificationAction,
  InAppNotificationProvider,
  InconsistentVersionNotificationProvider,
  LockdownModeNotificationProvider,
  ReconnectingNotificationProvider,
  UnsupportedVersionNotificationProvider,
} from '../../shared/notifications';
import { RoutePath } from '../../shared/routes';
import { useAppContext } from '../context';
import {
  useAppUpgradeDownloadProgressValue,
  useAppUpgradeEventType,
  useHasAppUpgradeError,
} from '../hooks';
import useActions from '../lib/actionsHook';
import { Button } from '../lib/components';
import { TransitionType, useHistory } from '../lib/history';
import {
  AppUpgradeErrorNotificationProvider,
  AppUpgradeProgressNotificationProvider,
  AppUpgradeReadyNotificationProvider,
  NewDeviceNotificationProvider,
  NewVersionNotificationProvider,
  UnsupportedWireGuardPortNotificationProvider,
} from '../lib/notifications';
import { AppUpgradeAvailableNotificationProvider } from '../lib/notifications/app-upgrade-available';
import { useMounted } from '../lib/utility-hooks';
import accountActions from '../redux/account/actions';
import { convertEventTypeToStep } from '../redux/app-upgrade/helpers';
import { useAppUpgradeError, useVersionSuggestedUpgrade } from '../redux/hooks';
import { IReduxState, useSelector } from '../redux/store';
import { ModalAlert, ModalAlertType, ModalMessage, ModalMessageList } from './Modal';
import {
  NotificationActions,
  NotificationBanner,
  NotificationCloseAction,
  NotificationContent,
  NotificationIndicator,
  NotificationOpenLinkAction,
  NotificationTitle,
  NotificationTroubleshootDialogAction,
} from './NotificationBanner';
import { NotificationSubtitle } from './NotificationSubtitle';

interface IProps {
  className?: string;
}

export default function NotificationArea(props: IProps) {
  const { showFullDiskAccessSettings } = useAppContext();

  const account = useSelector((state: IReduxState) => state.account);
  const locale = useSelector((state: IReduxState) => state.userInterface.locale);
  const tunnelState = useSelector((state: IReduxState) => state.connection.status);
  const connection = useSelector((state: IReduxState) => state.connection);
  const version = useSelector((state: IReduxState) => state.version);
  const allowedPortRanges = useSelector((state) => state.settings.wireguardEndpointData.portRanges);
  const relaySettings = useSelector((state) => state.settings.relaySettings);

  const lockdownModeSetting = useSelector((state: IReduxState) => state.settings.lockdownMode);
  const hasExcludedApps = useSelector(
    (state: IReduxState) =>
      state.settings.splitTunneling && state.settings.splitTunnelingApplications.length > 0,
  );

  const { hideNewDeviceBanner } = useActions(accountActions);

  const { setDisplayedChangelog, setDismissedUpgrade, appUpgrade, appUpgradeInstallerStart } =
    useAppContext();

  const currentVersion = useSelector((state) => state.version.current);
  const displayedForVersion = useSelector(
    (state) => state.settings.guiSettings.changelogDisplayedForVersion,
  );
  const changelog = useSelector((state) => state.userInterface.changelog);

  const close = useCallback(() => {
    setDisplayedChangelog();
  }, [setDisplayedChangelog]);

  const [isModalOpen, setIsModalOpen] = useState(false);

  const { setSplitTunnelingState } = useAppContext();
  const disableSplitTunneling = useCallback(async () => {
    setIsModalOpen(false);
    await setSplitTunnelingState(false);
  }, [setSplitTunnelingState]);

  const updateDismissedForVersion = useSelector(
    (state) => state.settings.guiSettings.updateDismissedForVersion,
  );
  const hasAppUpgradeError = useHasAppUpgradeError();
  const { error } = useAppUpgradeError();

  const restartAppUpgrade = useCallback(() => {
    appUpgrade();
  }, [appUpgrade]);
  const restartAppUpgradeInstaller = useCallback(() => {
    appUpgradeInstallerStart();
  }, [appUpgradeInstallerStart]);

  const { suggestedUpgrade } = useVersionSuggestedUpgrade();

  const appUpgradeDownloadProgressValue = useAppUpgradeDownloadProgressValue();
  const appUpgradeEventType = useAppUpgradeEventType();
  const appUpgradeStep = convertEventTypeToStep(appUpgradeEventType);

  const notificationProviders: InAppNotificationProvider[] = [
    new ConnectingNotificationProvider({ tunnelState }),
    new ReconnectingNotificationProvider(tunnelState),
    new LockdownModeNotificationProvider({
      tunnelState,
      lockdownModeSetting,
      hasExcludedApps,
    }),
    new AppUpgradeErrorNotificationProvider({
      hasAppUpgradeError,
      appUpgradeError: error,
      restartAppUpgrade,
      restartAppUpgradeInstaller,
    }),
    new AppUpgradeReadyNotificationProvider({
      appUpgradeEventType,
      suggestedUpgradeVersion: suggestedUpgrade?.version,
    }),
    new AppUpgradeProgressNotificationProvider({
      appUpgradeStep,
      appUpgradeEventType,
      appUpgradeDownloadProgressValue,
    }),
    new UnsupportedWireGuardPortNotificationProvider({
      connection,
      relaySettings,
      allowedPortRanges,
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
      close,
    }),
    new AppUpgradeAvailableNotificationProvider({
      platform: window.env.platform,
      suggestedUpgradeVersion: suggestedUpgrade?.version,
      suggestedIsBeta: version.suggestedIsBeta,
      updateDismissedForVersion,
      close: setDismissedUpgrade,
    }),
  );

  const notificationProvider = notificationProviders.find((notification) =>
    notification.mayDisplay(),
  );

  const notification = notificationProvider?.getInAppNotification();
  if (notificationProvider) {
    if (!notification) {
      log.error(
        `Notification providers mayDisplay() returned true but getInAppNotification() returned undefined for ${notificationProvider.constructor.name}`,
      );
    }
  }

  // We only want to animate notifications after first mount,
  // so as to prevent an animation from animating in when the
  // app has just started.
  const mounted = useMounted();
  const isMounted = mounted();

  return (
    <AnimatePresence>
      {notification && (
        <NotificationBanner
          animateIn={isMounted}
          aria-hidden={!notification}
          className={props.className}>
          <NotificationIndicator
            $type={notification.indicator}
            data-testid="notificationIndicator"
          />
          <NotificationContent role="status" aria-live="polite">
            <NotificationTitle data-testid="notificationTitle">
              {notification.title}
            </NotificationTitle>
            <NotificationSubtitle
              data-testid="notificationSubTitle"
              subtitle={notification.subtitle}
            />
          </NotificationContent>
          {notification.action && (
            <NotificationActionWrapper
              action={notification.action}
              isModalOpen={isModalOpen}
              setIsModalOpen={setIsModalOpen}
            />
          )}
        </NotificationBanner>
      )}
    </AnimatePresence>
  );
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

  const closeTroubleshootModal = useCallback(() => setIsModalOpen(false), [setIsModalOpen]);

  const handleClick = useCallback(() => {
    if (action) {
      switch (action.type) {
        case 'navigate-external':
          if (action.link.withAuth) {
            return openUrlWithAuth(action.link.to);
          } else {
            return openUrl(action.link.to);
          }
        case 'troubleshoot-dialog':
          setIsModalOpen(true);
          break;
        case 'close':
          action.close();
          break;
      }
    }

    return Promise.resolve();
  }, [action, setIsModalOpen, openUrlWithAuth, openUrl]);

  const goToProblemReport = useCallback(() => {
    closeTroubleshootModal();
    push(RoutePath.problemReport, { transition: TransitionType.show });
  }, [closeTroubleshootModal, push]);

  let actionComponent: React.ReactElement | undefined;
  if (action) {
    switch (action.type) {
      case 'navigate-external':
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
    <Button key="problem-report" onClick={goToProblemReport}>
      <Button.Text>
        {
          // TRANSLATORS: Button label to send a problem report.
          messages.pgettext('in-app-notifications', 'Send problem report')
        }
      </Button.Text>
    </Button>
  ) : (
    <Button variant="success" key="problem-report" onClick={goToProblemReport}>
      <Button.Text>
        {
          // TRANSLATORS: Button label to send a problem report.
          messages.pgettext('in-app-notifications', 'Send problem report')
        }
      </Button.Text>
    </Button>
  );

  let buttons = [
    problemReportButton,
    <Button key="back" onClick={closeTroubleshootModal}>
      <Button.Text>{messages.gettext('Back')}</Button.Text>
    </Button>,
  ];

  if (action.troubleshoot?.buttons) {
    const actionButtons = action.troubleshoot.buttons.map(({ variant, label, action }) => (
      <Button key={label} variant={variant} onClick={action}>
        <Button.Text>{label}</Button.Text>
      </Button>
    ));

    buttons = actionButtons.concat(buttons);
  }

  return (
    <>
      <NotificationActions>{actionComponent}</NotificationActions>
      <ModalAlert
        isOpen={isModalOpen}
        type={ModalAlertType.info}
        buttons={buttons}
        close={closeTroubleshootModal}>
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
