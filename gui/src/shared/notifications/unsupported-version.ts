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
    return {
      message: this.getMessage(),
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
    return {
      indicator: 'error',
      title: messages.pgettext('in-app-notifications', 'UNSUPPORTED VERSION'),
      subtitle: this.getMessage(),
      action: { type: 'open-url', url: links.download },
    };
  }

  private getMessage(): string {
    // TRANSLATORS: The in-app banner and system notification which are displayed to the user when the running app becomes unsupported.
    return messages.pgettext(
      'notifications',
      'Your privacy might be at risk with this unsupported app version. Please update now.',
    );
  }
}
