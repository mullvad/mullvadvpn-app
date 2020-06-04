import { sprintf } from 'sprintf-js';
import { links } from '../../config.json';
import { messages } from '../../shared/gettext';
import {
  InAppNotification,
  NotificationAction,
  InAppNotificationIndicatorType,
  NotificationProvider,
  SystemNotification,
} from './notification';

interface UnsupportedVersionNotificationContext {
  supported: boolean;
  consistent: boolean;
  nextUpgrade: string | null;
}

export class UnsupportedVersionNotificationProvider
  extends NotificationProvider<UnsupportedVersionNotificationContext>
  implements SystemNotification, InAppNotification {
  public get visible() {
    return this.context.consistent && !this.context.supported && this.context.nextUpgrade !== null;
  }

  public action: NotificationAction = { type: 'open-url', url: links.download };

  public get message() {
    return sprintf(
      // TRANSLATORS: The system notification displayed to the user when the running app becomes unsupported.
      // TRANSLATORS: Available placeholder:
      // TRANSLATORS: %(version) - the newest available version of the app
      messages.pgettext(
        'notifications',
        'You are running an unsupported app version. Please upgrade to %(version)s now to ensure your security',
      ),
      { version: this.context.nextUpgrade },
    );
  }

  public critical = true;

  public presentOnce = true;

  public suppressInDevelopment = true;

  public indicator: InAppNotificationIndicatorType = 'error';

  public title = messages.pgettext('in-app-notifications', 'UNSUPPORTED VERSION');

  public get body() {
    return sprintf(
      // TRANSLATORS: The in-app banner displayed to the user when the running app becomes unsupported.
      // TRANSLATORS: Available placeholders:
      // TRANSLATORS: %(version)s - the newest available version of the app
      messages.pgettext(
        'in-app-notifications',
        'You are running an unsupported app version. Please upgrade to %(version)s now to ensure your security',
      ),
      { version: this.context.nextUpgrade },
    );
  }
}
