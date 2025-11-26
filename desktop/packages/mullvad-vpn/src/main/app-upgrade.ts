import { spawn } from 'child_process';
import fs from 'fs/promises';
import { tmpdir } from 'os';

import { DaemonAppUpgradeEvent } from '../shared/daemon-rpc-types';
import log from '../shared/logging';
import { DaemonRpc, SubscriptionListener } from './daemon-rpc';
import { IpcMainEventChannel } from './ipc-event-channel';

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
        await this.checkInstallerPath(verifiedInstallerPath);
        this.startInstaller(verifiedInstallerPath);
      } catch (e) {
        const error = e as Error;
        log.error(
          `An error occurred when trying to start the installer: ${verifiedInstallerPath}. Error: ${error.message}`,
        );
      }
    });

    IpcMainEventChannel.app.handleGetUpgradeCacheDir(() => this.daemonRpc.getAppUpgradeCacheDir());
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

  private async checkInstallerPath(verifiedInstallerPath: string) {
    try {
      // fs.stat throws if the path does not exist
      const stat = await fs.stat(verifiedInstallerPath);
      // If the path exists, verify that its a file.
      if (!stat.isFile()) {
        throw new Error('Verified installer path is not a file.');
      }
    } catch (e) {
      // Let the render process know we encountered an error
      IpcMainEventChannel.app.notifyUpgradeError?.('GENERAL_ERROR');
      // If the daemon for some reason doesn't reply with an aborted event we should
      // let the render process know which event step to re-start from.
      IpcMainEventChannel.app.notifyUpgradeEvent?.({
        type: 'APP_UPGRADE_STATUS_MANUAL_START_INSTALLER',
      });

      // Let the daemon know the we are aborting the upgrade
      this.daemonRpc.appUpgradeAbort();

      const error = e as Error;
      throw new Error(
        `An error occurred when checking installer at path: ${verifiedInstallerPath}. Error: ${error.message}`,
      );
    }
  }

  private spawnChildMac(verifiedInstallerPath: string) {
    const child = spawn('open', [verifiedInstallerPath, '--wait-apps'], {
      detached: true,
    });

    return child;
  }

  private spawnChildWindows(verifiedInstallerPath: string) {
    const SYSTEM_ROOT_PATH = process.env.SYSTEMROOT || process.env.windir || 'C:\\Windows';
    const CMD_PATH = `${SYSTEM_ROOT_PATH}\\System32\\cmd.exe`;
    const quotedVerifiedInstallerPath = `"${verifiedInstallerPath}"`;
    const updaterFlag = '/inapp';

    const cwd = tmpdir();
    const child = spawn(CMD_PATH, ['/C', 'start', '""', quotedVerifiedInstallerPath, updaterFlag], {
      cwd,
      detached: true,
      stdio: 'ignore',
      windowsVerbatimArguments: true,
    });

    return child;
  }

  private spawnChild(verifiedInstallerPath: string) {
    if (process.platform === 'darwin') {
      return this.spawnChildMac(verifiedInstallerPath);
    }

    if (process.platform === 'win32') {
      return this.spawnChildWindows(verifiedInstallerPath);
    }

    throw new Error(`Unsupported platform: ${process.platform}`);
  }

  private startInstaller(verifiedInstallerPath: string) {
    try {
      log.info(`Starting verified installer at path: ${verifiedInstallerPath}`);
      const child = this.spawnChild(verifiedInstallerPath);
      IpcMainEventChannel.app.notifyUpgradeEvent?.({
        type: 'APP_UPGRADE_STATUS_STARTED_INSTALLER',
      });

      child.once('error', (error) => {
        log.error(`An error occurred with the installer: ${error.message}`);
        IpcMainEventChannel.app.notifyUpgradeError?.('INSTALLER_FAILED');
        IpcMainEventChannel.app.notifyUpgradeEvent?.({
          type: 'APP_UPGRADE_STATUS_EXITED_INSTALLER',
        });
      });

      child.once('exit', (code) => {
        if (code !== 0) {
          log.error(`The installer exited unexpectedly with exit code: ${code}`);
          IpcMainEventChannel.app.notifyUpgradeError?.('INSTALLER_FAILED');
        }

        IpcMainEventChannel.app.notifyUpgradeEvent?.({
          type: 'APP_UPGRADE_STATUS_EXITED_INSTALLER',
        });
      });
    } catch (e) {
      IpcMainEventChannel.app.notifyUpgradeError?.('START_INSTALLER_FAILED');
      IpcMainEventChannel.app.notifyUpgradeEvent?.({
        type: 'APP_UPGRADE_STATUS_EXITED_INSTALLER',
      });

      const error = e as Error;
      log.error(
        `An error occurred when starting installer at path: ${verifiedInstallerPath}. Error: ${error.message}`,
      );
    }
  }
}
