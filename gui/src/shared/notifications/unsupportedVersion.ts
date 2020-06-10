import { sprintf } from 'sprintf-js';
import { links } from '../../config.json';
import { messages } from '../../shared/gettext';
import {
  InAppNotification,
  SystemNotification,
  InAppNotificationProvider,
  SystemNotificationProvider,
} from './notification';

interface UnsupportedVersionNotificationContext {
  supported: boolean;
  consistent: boolean;
  nextUpgrade: string | null;
}

export class UnsupportedVersionNotificationProvider
  implements SystemNotificationProvider, InAppNotificationProvider {
  public constructor(private context: UnsupportedVersionNotificationContext) {}

  public mayDisplay() {
    return this.context.consistent && !this.context.supported && this.context.nextUpgrade !== null;
  }

  public getSystemNotification(): SystemNotification {
    const message = sprintf(
      // TRANSLATORS: The system notification displayed to the user when the running app becomes unsupported.
      // TRANSLATORS: Available placeholder:
      // TRANSLATORS: %(version) - the newest available version of the app
      messages.pgettext(
        'notifications',
        'You are running an unsupported app version. Please upgrade to %(version)s now to ensure your security',
      ),
      { version: this.context.nextUpgrade },
    );

    return {
      message,
      critical: true,
      action: { type: 'open-url', url: links.download },
      presentOnce: { value: true, name: this.constructor.name },
      suppressInDevelopment: true,
    };
  }

  public getInAppNotification(): InAppNotification {
    const subtitle = sprintf(
      // TRANSLATORS: The in-app banner displayed to the user when the running app becomes unsupported.
      // TRANSLATORS: Available placeholders:
      // TRANSLATORS: %(version)s - the newest available version of the app
      messages.pgettext(
        'in-app-notifications',
        'You are running an unsupported app version. Please upgrade to %(version)s now to ensure your security',
      ),
      { version: this.context.nextUpgrade },
    );

    return {
      indicator: 'error',
      title: messages.pgettext('in-app-notifications', 'UNSUPPORTED VERSION'),
      subtitle,
      action: { type: 'open-url', url: links.download },
    };
  }
}
