import { RoutePath } from '../../renderer/lib/routes';
import { messages } from '../gettext';
import { IChangelog } from '../ipc-types';
import { InAppNotification, InAppNotificationProvider } from './notification';

interface NewVersionNotificationContext {
  currentVersion: string;
  displayedForVersion: string;
  changelog: IChangelog;
  close: () => void;
}

export class NewVersionNotificationProvider implements InAppNotificationProvider {
  public constructor(private context: NewVersionNotificationContext) {}

  public mayDisplay = () => {
    return (
      this.context.displayedForVersion !== this.context.currentVersion &&
      this.context.changelog.length > 0
    );
  };

  public getInAppNotification(): InAppNotification {
    const title = messages.pgettext('in-app-notifications', 'NEW VERSION INSTALLED');
    const subtitle = messages.pgettext('in-app-notifications', "Click here to see what's new.");
    return {
      indicator: 'success',
      action: { type: 'close', close: this.context.close },
      title,
      subtitle,
      subtitleAction: {
        type: 'navigate',
        link: {
          to: RoutePath.changelog,
          onClick: this.context.close,
          'aria-label': messages.pgettext(
            'accessibility',
            'New version installed, click here to see the changelog',
          ),
        },
      },
    };
  }
}
