import { spawn } from 'child_process';

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

    IpcMainEventChannel.app.handleUpgradeInstallerStart((verifiedInstallerPath: string) => {
      try {
        this.startInstaller(verifiedInstallerPath);
        IpcMainEventChannel.app.notifyUpgradeEvent?.({
          type: 'APP_UPGRADE_STATUS_STARTED_INSTALLER',
        });
      } catch (e) {
        IpcMainEventChannel.app.notifyUpgradeEvent?.({
          type: 'APP_UPGRADE_STATUS_EXITED_INSTALLER',
        });
        IpcMainEventChannel.app.notifyUpgradeError?.('START_INSTALLER_FAILED');

        const error = e as Error;
        log.error(
          `An error occurred when starting installer at path: ${verifiedInstallerPath}. Error: ${error.message}`,
        );
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

  private spawnChildMac(verifiedInstallerPath: string) {
    const child = spawn('open', [verifiedInstallerPath, '--wait-apps'], {
      detached: true,
    });

    return child;
  }

  private spawnChildWindows(verifiedInstallerPath: string) {
    const SYSTEM_ROOT_PATH = process.env.SYSTEMROOT || process.env.windir || 'C:\\Windows';
    const POWERSHELL_PATH = `${SYSTEM_ROOT_PATH}\\System32\\WindowsPowerShell\\v1.0\\powershell`;
    const quotedVerifiedInstallerPath = `'${verifiedInstallerPath}'`;

    const child = spawn(POWERSHELL_PATH, ['Start', '-Wait', quotedVerifiedInstallerPath], {
      detached: true,
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
    const child = this.spawnChild(verifiedInstallerPath);

    child.on('error', (error) => {
      log.error(`An error occurred with the installer: ${error.message}`);
      IpcMainEventChannel.app.notifyUpgradeError?.('INSTALLER_FAILED');
      IpcMainEventChannel.app.notifyUpgradeEvent?.({
        type: 'APP_UPGRADE_STATUS_EXITED_INSTALLER',
      });
    });

    child.on('exit', (code) => {
      if (code !== 0) {
        log.error(`The installer exited unexpectedly with exit code: ${code}`);
        IpcMainEventChannel.app.notifyUpgradeError?.('INSTALLER_FAILED');
      }

      IpcMainEventChannel.app.notifyUpgradeEvent?.({
        type: 'APP_UPGRADE_STATUS_EXITED_INSTALLER',
      });
    });
  }
}
