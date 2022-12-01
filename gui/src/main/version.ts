import { app } from 'electron';

import { IAppVersionInfo } from '../shared/daemon-rpc-types';
import { ICurrentAppVersionInfo } from '../shared/ipc-types';
import log from '../shared/logging';
import {
  InconsistentVersionNotificationProvider,
  SystemNotificationCategory,
  UnsupportedVersionNotificationProvider,
  UpdateAvailableNotificationProvider,
} from '../shared/notifications/notification';
import { DaemonRpc } from './daemon-rpc';
import { IpcMainEventChannel } from './ipc-event-channel';
import { NotificationSender } from './notification-controller';

const GUI_VERSION = app.getVersion().replace('.0', '');
/// Mirrors the beta check regex in the daemon. Matches only well formed beta versions
const IS_BETA = /^(\d{4})\.(\d+)-beta(\d+)$/;

export default class Version {
  private currentVersionData: ICurrentAppVersionInfo = {
    daemon: undefined,
    gui: GUI_VERSION,
    isConsistent: true,
    isBeta: IS_BETA.test(GUI_VERSION),
  };

  private upgradeVersionData: IAppVersionInfo = {
    supported: true,
    suggestedUpgrade: undefined,
  };

  public constructor(
    private delegate: NotificationSender,
    private daemonRpc: DaemonRpc,
    private updateNotificationDisabled: boolean,
  ) {}

  public get currentVersion() {
    return this.currentVersionData;
  }

  public get upgradeVersion() {
    return this.upgradeVersionData;
  }

  public setDaemonVersion(daemonVersion: string) {
    const versionInfo = {
      ...this.currentVersionData,
      daemon: daemonVersion,
      isConsistent: daemonVersion === this.currentVersionData.gui,
    };

    this.currentVersionData = versionInfo;

    if (!versionInfo.isConsistent) {
      log.info('Inconsistent version', {
        guiVersion: versionInfo.gui,
        daemonVersion: versionInfo.daemon,
      });
    }

    // notify user about inconsistent version
    const notificationProvider = new InconsistentVersionNotificationProvider({
      consistent: versionInfo.isConsistent,
    });
    if (notificationProvider.mayDisplay()) {
      this.delegate.notify(notificationProvider.getSystemNotification());
    } else {
      this.delegate.closeNotificationsInCategory(SystemNotificationCategory.inconsistentVersion);
    }

    // notify renderer
    IpcMainEventChannel.currentVersion.notify?.(versionInfo);
  }

  public setLatestVersion(latestVersionInfo: IAppVersionInfo) {
    if (this.updateNotificationDisabled) {
      return;
    }

    const suggestedIsBeta =
      latestVersionInfo.suggestedUpgrade !== undefined &&
      IS_BETA.test(latestVersionInfo.suggestedUpgrade);

    const upgradeVersion = {
      ...latestVersionInfo,
      suggestedIsBeta,
    };

    this.upgradeVersionData = upgradeVersion;

    // notify user to update the app if it became unsupported
    const notificationProviders = [
      new UnsupportedVersionNotificationProvider({
        supported: latestVersionInfo.supported,
        consistent: this.currentVersionData.isConsistent,
        suggestedUpgrade: latestVersionInfo.suggestedUpgrade,
        suggestedIsBeta,
      }),
      new UpdateAvailableNotificationProvider({
        suggestedUpgrade: latestVersionInfo.suggestedUpgrade,
        suggestedIsBeta,
      }),
    ];
    const notificationProvider = notificationProviders.find((notificationProvider) =>
      notificationProvider.mayDisplay(),
    );
    if (notificationProvider) {
      this.delegate.notify(notificationProvider.getSystemNotification());
    } else {
      this.delegate.closeNotificationsInCategory(SystemNotificationCategory.newVersion);
    }

    IpcMainEventChannel.upgradeVersion.notify?.(upgradeVersion);
  }

  public async fetchLatestVersion() {
    try {
      this.setLatestVersion(await this.daemonRpc.getVersionInfo());
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to request the version info: ${error.message}`);
    }
  }
}
