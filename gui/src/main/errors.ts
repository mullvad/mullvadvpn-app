import { messages } from '../shared/gettext';

export class InvalidAccountError extends Error {
  constructor() {
    super(
      // TRANSLATORS: Error message shown above login input when trying to login with a non-existent
      // TRANSLATORS: account number.
      messages.pgettext('login-view', 'Invalid account number'),
    );
  }
}

export class CommunicationError extends Error {
  constructor() {
    super('api.mullvad.net is blocked, please check your firewall');
  }
}

export class TooManyDevicesError extends Error {
  constructor() {
    super(
      // TRANSLATORS: Error message shown above login input when trying to login to an account with
      // TRANSLATORS: too many registered devices.
      messages.pgettext('login-view', 'Too many devices'),
    );
  }
}

export class ListDevicesError extends Error {
  constructor() {
    super(
      // TRANSLATORS: Error message shown above login input when trying to login but the app fails
      // TRANSLATORS: to fetch the list of registered devices.
      messages.pgettext('login-view', 'Failed to fetch list of devices'),
    );
  }
}
