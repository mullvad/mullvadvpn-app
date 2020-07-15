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
  suggestedUpgrade?: string;
}

export class UnsupportedVersionNotificationProvider
  implements SystemNotificationProvider, InAppNotificationProvider {
  public constructor(private context: UnsupportedVersionNotificationContext) {}

  public mayDisplay() {
    return this.context.consistent && !this.context.supported;
  }

  public getSystemNotification(): SystemNotification {
    const message = this.getMessage();
    return {
      message,
      critical: true,
      action: {
        type: 'open-url',
        url: links.download,
        text: messages.pgettext('notifications', 'Upgrade'),
      },
      presentOnce: { value: true, name: this.constructor.name },
      suppressInDevelopment: true,
    };
  }

  public getInAppNotification(): InAppNotification {
    const subtitle = this.getMessage();

    return {
      indicator: 'error',
      title: messages.pgettext('in-app-notifications', 'UNSUPPORTED VERSION'),
      subtitle,
      action: { type: 'open-url', url: links.download },
    };
  }

  private getMessage(): string {
    // TRANSLATORS: The in-app banner and system notification which are displayed to the user when the running app becomes unsupported.
    let message = messages.pgettext('notifications', 'You are running an unsupported app version.');
    if (this.context.suggestedUpgrade) {
      message += ' ';
      message += sprintf(
        // TRANSLATORS: Appendix to the system notification and in-app banner about the app becoming unsupported with the suggested supported version.
        // TRANSLATORS: Available placeholder:
        // TRANSLATORS: %(version) - the newest available version of the app
        messages.pgettext(
          'notifications',
          'Please upgrade to %(version)s now to ensure your security',
        ),
        { version: this.context.suggestedUpgrade },
      );
    }
    return message;
  }
}
