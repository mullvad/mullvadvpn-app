import { execFile } from 'child_process';
import fs from 'fs/promises';
import { promisify } from 'util';

import { DaemonAppUpgradeEvent } from '../shared/daemon-rpc-types';
import log from '../shared/logging';
import { DaemonRpc, SubscriptionListener } from './daemon-rpc';
import { IpcMainEventChannel } from './ipc-event-channel';

const execFilePromise = promisify(execFile);

export default class AppUpgrade {
  public constructor(private daemonRpc: DaemonRpc) {}

  public registerIpcListeners() {
    IpcMainEventChannel.app.handleUpgrade(() => {
      this.daemonRpc.appUpgrade();
    });

    IpcMainEventChannel.app.handleUpgradeAbort(() => {
      this.daemonRpc.appUpgradeAbort();
    });

    IpcMainEventChannel.app.handleUpgradeInstallerStart(async (verifiedInstallerPath: string) => {
      try {
        await this.startInstaller(verifiedInstallerPath);
        IpcMainEventChannel.app.notifyUpgradeEvent?.({
          type: 'APP_UPGRADE_STATUS_STARTED_INSTALLER',
        });
      } catch {
        IpcMainEventChannel.app.notifyUpgradeError?.('START_INSTALLER_FAILED');
      }
    });
  }

  public subscribeEvents() {
    const daemonAppUpgradeEventListener = new SubscriptionListener(
      (appUpgradeEvent: DaemonAppUpgradeEvent) => {
        if (appUpgradeEvent.type === 'APP_UPGRADE_ERROR') {
          IpcMainEventChannel.app.notifyUpgradeError?.(appUpgradeEvent.error);
        } else {
          IpcMainEventChannel.app.notifyUpgradeEvent?.(appUpgradeEvent);
        }
      },
      (error: Error) => {
        log.error(`Cannot deserialize the app upgrade event: ${error.message}`);
      },
    );

    this.daemonRpc.subscribeAppUpgradeEventListener(daemonAppUpgradeEventListener);

    return daemonAppUpgradeEventListener;
  }

  private async startInstaller(verifiedInstallerPath: string) {
    await this.isInstallerExecutable(verifiedInstallerPath);
    await this.executeInstaller(verifiedInstallerPath);
  }

  private async executeInstaller(verifiedInstallerPath: string) {
    try {
      await execFilePromise(verifiedInstallerPath);
    } catch {
      throw new Error(`Could not start installer at path: ${verifiedInstallerPath}`);
    }
  }

  private async isInstallerExecutable(verifiedInstallerPath: string) {
    try {
      await fs.access(verifiedInstallerPath, fs.constants.X_OK);
    } catch {
      throw new Error(`An executable installer is not available at path: ${verifiedInstallerPath}`);
    }
  }
}
