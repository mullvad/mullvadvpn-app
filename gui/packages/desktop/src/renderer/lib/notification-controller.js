// @flow

import { remote } from 'electron';

export default class NotificationController {
  _activeNotification: ?Notification;

  show(message: string) {
    const lastNotification = this._activeNotification;
    const sameAsLastNotification = lastNotification && lastNotification.body === message;

    if (sameAsLastNotification || remote.getCurrentWindow().isVisible()) {
      return;
    }

    const newNotification = new Notification(remote.app.getName(), { body: message, silent: true });

    this._activeNotification = newNotification;

    newNotification.addEventListener('show', () => {
      // If the notification is closed too soon, it might still get shown. If that happens, close()
      // should be called again so that it is closed immediately.
      if (this._activeNotification !== newNotification) {
        newNotification.close();
      }
    });

    if (lastNotification) {
      lastNotification.close();
    }
  }
}
