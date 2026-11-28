import * as grpc from '@grpc/grpc-js';
import { promises as fs } from 'fs';

import { ImportConfigResponse, IWireguardIdentity } from '../shared/daemon-rpc-types';
import log from '../shared/logging';
import { DaemonRpc } from './daemon-rpc';
import { IpcMainEventChannel } from './ipc-event-channel';

// A WireGuard config is a few hundred bytes. Refuse to read anything unreasonably large.
const MAX_CONFIG_FILE_SIZE = 64 * 1024;

export interface IdentityDelegate {
  onIdentityChanged(): void;
}

/**
 * Keeps track of the WireGuard identity (private key and tunnel addresses imported from a config
 * generated at mullvad.net). This replaces the former account and device handling: the app does not
 * know about accounts, it only needs a config to be able to connect.
 */
export default class Identity {
  private identityValue?: IWireguardIdentity = undefined;

  public constructor(
    private delegate: IdentityDelegate,
    private daemonRpc: DaemonRpc,
  ) {}

  public get identity() {
    return this.identityValue;
  }

  public hasIdentity(): boolean {
    return this.identityValue !== undefined;
  }

  public registerIpcListeners() {
    IpcMainEventChannel.identity.handleImportText((config) => this.importConfig(config));
    IpcMainEventChannel.identity.handleImportFile((path) => this.importConfigFile(path));
    IpcMainEventChannel.identity.handleClear(() => this.daemonRpc.clearWireguardIdentity());
  }

  /** Called when the daemon reports that the identity was imported, replaced or cleared. */
  public handleIdentityEvent(identity?: IWireguardIdentity) {
    this.identityValue = identity;
    this.delegate.onIdentityChanged();
    IpcMainEventChannel.identity.notify?.(identity);
  }

  public async fetch(): Promise<void> {
    try {
      this.handleIdentityEvent(await this.daemonRpc.getWireguardIdentity());
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to fetch the WireGuard identity: ${error.message}`);
    }
  }

  private async importConfigFile(path: string): Promise<ImportConfigResponse> {
    let config: string;
    try {
      const stat = await fs.stat(path);
      if (!stat.isFile() || stat.size > MAX_CONFIG_FILE_SIZE) {
        return { type: 'error', error: 'invalid-config', message: 'Not a WireGuard config file' };
      }
      config = await fs.readFile(path, { encoding: 'utf8' });
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to read the config file: ${error.message}`);
      return { type: 'error', error: 'file-read' };
    }

    return this.importConfig(config);
  }

  private async importConfig(config: string): Promise<ImportConfigResponse> {
    try {
      // The config contains a private key, so it must never be logged.
      const identity = await this.daemonRpc.importWireguardConfig(config);
      return { type: 'success', identity };
    } catch (e) {
      const error = e as grpc.ServiceError;
      if (error.code === grpc.status.INVALID_ARGUMENT) {
        log.error('The imported WireGuard config was rejected by the daemon');
        return { type: 'error', error: 'invalid-config', message: error.details };
      }

      log.error(`Failed to import the WireGuard config: ${error.message}`);
      return { type: 'error', error: 'communication' };
    }
  }
}
