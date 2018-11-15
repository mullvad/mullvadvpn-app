// @flow

import BaseSubscriptionProxy from './base-proxy';
import { SubscriptionListener } from '../daemon-rpc-proxy';
import type { DaemonRpcProtocol, TunnelStateTransition } from '../daemon-rpc-proxy';

export default class TunnelStateProxy extends BaseSubscriptionProxy<TunnelStateTransition> {
  static subscribeValueListener(
    rpc: DaemonRpcProtocol,
    listener: SubscriptionListener<TunnelStateTransition>,
  ) {
    return rpc.subscribeStateListener(listener);
  }

  static requestValue(rpc: DaemonRpcProtocol): Promise<TunnelStateTransition> {
    return rpc.getState();
  }
}
