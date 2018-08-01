// @flow
import { remote } from 'electron';

export default class NotificationController {
  _activeNotification: ?Notification;

  show(message: string) {
    const lastNotification = this._activeNotification;
    const newNotification = new Notification(remote.app.getName(), { body: message, silent: true });

    this._activeNotification = newNotification;

    newNotification.addEventListener('show', () => {
      if (this._activeNotification !== newNotification) {
        newNotification.close();
      }
    });

    if (lastNotification) {
      lastNotification.close();
    }
  }
}
