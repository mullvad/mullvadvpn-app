import { sprintf } from 'sprintf-js';

import { strings, urls } from '../../../shared/constants';
import { TunnelProtocol } from '../../../shared/daemon-rpc-types';
import { messages } from '../../../shared/gettext';
import { InAppNotification, InAppNotificationProvider } from '../../../shared/notifications';

interface OpenVpnSupportEndingNotificationContext {
  tunnelProtocol: TunnelProtocol;
}

export class OpenVpnSupportEndingNotificationProvider implements InAppNotificationProvider {
  public constructor(private context: OpenVpnSupportEndingNotificationContext) {}

  public mayDisplay = () => {
    return this.context.tunnelProtocol === 'openvpn';
  };

  public getInAppNotification(): InAppNotification {
    const capitalizedOpenVpn = strings.openvpn.toUpperCase();
    return {
      indicator: 'warning',
      title: sprintf(
        // TRANSLATORS: Notification title indicating that OpenVPN support is ending.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(openVpn)s - Will be replaced with OPENVPN
        messages.pgettext('in-app-notifications', '%(openVpn)s SUPPORT IS ENDING'),
        {
          openVpn: capitalizedOpenVpn,
        },
      ),
      subtitle: [
        {
          content: sprintf(
            // TRANSLATORS: Notification subtitle indicating that OpenVPN support is ending.
            // TRANSLATORS: Available placeholders:
            // TRANSLATORS: %(wireGuard)s - Will be replaced with WireGuard
            messages.pgettext(
              'in-app-notifications',
              'Please change tunnel protocol to %(wireGuard)s',
            ),
            { wireGuard: strings.wireguard },
          ),
        },
        {
          content:
            // TRANSLATORS: Link in notication to a blog post about OpenVPN support ending.
            messages.pgettext('in-app-notifications', 'Read more'),
          action: {
            type: 'navigate-external',
            link: {
              to: urls.removingOpenVpnBlog,
              'aria-label':
                // TRANSLATORS: Accessibility label for link to blog post about OpenVPN support ending.
                messages.pgettext(
                  'accessibility',
                  'Go to blog post to read more, opens externally',
                ),
            },
          },
        },
      ],
    };
  }
}
