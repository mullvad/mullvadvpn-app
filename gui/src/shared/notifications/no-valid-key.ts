import { WgKeyState } from '../../renderer/redux/settings/reducers';
import { messages } from '../../shared/gettext';
import { LiftedConstraint, TunnelProtocol } from '../daemon-rpc-types';
import { InAppNotification, InAppNotificationProvider } from './notification';

interface NoValidKeyNotificationContext {
  tunnelProtocol?: LiftedConstraint<TunnelProtocol>;
  wireGuardKey: WgKeyState;
}

export class NoValidKeyNotificationProvider implements InAppNotificationProvider {
  public constructor(private context: NoValidKeyNotificationContext) {}

  public mayDisplay() {
    const usingWireGuard =
      this.context.tunnelProtocol === 'wireguard' ||
      (this.context.tunnelProtocol === 'any' && process.platform !== 'win32');
    const keyInvalid =
      this.context.wireGuardKey.type === 'key-not-set' ||
      this.context.wireGuardKey.type === 'too-many-keys' ||
      this.context.wireGuardKey.type === 'generation-failure' ||
      (this.context.wireGuardKey.type === 'key-set' &&
        this.context.wireGuardKey.key.valid === false) ||
      (this.context.wireGuardKey.type === 'key-set' &&
        this.context.wireGuardKey.key.replacementFailure === 'too_many_keys');

    return usingWireGuard && keyInvalid;
  }

  public getInAppNotification(): InAppNotification {
    return {
      indicator: 'warning',
      title: messages.pgettext('in-app-notifications', 'VALID WIREGUARD KEY IS MISSING'),
      subtitle: messages.pgettext('in-app-notifications', 'Manage keys under Advanced settings.'),
    };
  }
}
