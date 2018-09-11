// @flow

import BaseSubscriptionProxy from './base-proxy';
import { SubscriptionListener } from '../daemon-rpc';
import type { DaemonRpcProtocol, Settings } from '../daemon-rpc';

export default class SettingsProxy extends BaseSubscriptionProxy<Settings> {
  static subscribeValueListener(rpc: DaemonRpcProtocol, listener: SubscriptionListener<Settings>) {
    return rpc.subscribeSettingsListener(listener);
  }

  static requestValue(rpc: DaemonRpcProtocol): Promise<Settings> {
    return rpc.getSettings();
  }
}
