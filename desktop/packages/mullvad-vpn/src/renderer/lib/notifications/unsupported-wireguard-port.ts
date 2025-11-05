import { sprintf } from 'sprintf-js';

import { strings } from '../../../shared/constants';
import { liftConstraint, ObfuscationSettings } from '../../../shared/daemon-rpc-types';
import { messages } from '../../../shared/gettext';
import { InAppNotification, InAppNotificationProvider } from '../../../shared/notifications';
import { RoutePath } from '../../../shared/routes';
import { isInRanges } from '../../../shared/utils';
import { IConnectionReduxState } from '../../redux/connection/reducers';

interface UnsupportedWireGuardPortNotificationContext {
  connection: IConnectionReduxState;
  obfuscationSettings: ObfuscationSettings;
  allowedPortRanges: [number, number][];
}

export class UnsupportedWireGuardPortNotificationProvider implements InAppNotificationProvider {
  public constructor(private context: UnsupportedWireGuardPortNotificationContext) {}

  public mayDisplay = () => {
    const { connection, obfuscationSettings, allowedPortRanges } = this.context;
    if (connection.status.state === 'error') {
      const port = liftConstraint(obfuscationSettings.wireGuardPortSettings.port);
      if (port !== 'any' && !isInRanges(port, allowedPortRanges)) return true;
    }

    return false;
  };

  public getInAppNotification(): InAppNotification {
    return {
      indicator: 'error',
      title: messages.pgettext('in-app-notifications', 'BLOCKING INTERNET'),
      subtitle: [
        {
          content: sprintf(
            // TRANSLATORS: Notification subtitle indicating the user is using an unsupported port for WireGuard.
            // TRANSLATORS: Available placeholders:
            // TRANSLATORS: %(wireGuard)s - Will be replaced with WireGuard
            messages.pgettext(
              'in-app-notifications',
              'The selected %(wireGuard)s port is not supported, please change it under ',
            ),
            { wireGuard: strings.wireguard },
          ),
        },
        {
          content: sprintf(
            // TRANSLATORS: Link in notication to go to WireGuard settings.
            // TRANSLATORS: Available placeholders:
            // TRANSLATORS: %(wireGuard)s - Will be replaced with WireGuard
            messages.pgettext('in-app-notifications', '%(wireGuard)s settings.'),
            { wireGuard: strings.wireguard },
          ),
          action: {
            type: 'navigate-internal',
            link: {
              to: RoutePath.antiCensorship,
              'aria-label':
                // TRANSLATORS: Accessibility label for link to anti-censorship settings.
                messages.pgettext('accessibility', 'Go to anti-censorship settings.'),
            },
          },
        },
      ],
    };
  }
}
