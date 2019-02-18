import log from 'electron-log';
import { pgettext } from '../../shared/gettext';

export type AuthFailureKind =
  | 'INVALID_ACCOUNT'
  | 'EXPIRED_ACCOUNT'
  | 'TOO_MANY_CONNECTIONS'
  | 'UNKNOWN';

// These strings should match up with mullvad-types/src/auth_failed.rs

const GENERIC_FAILURE_MSG = pgettext('auth-failure', 'Account authentication failed.');

const INVALID_ACCOUNT_MSG = pgettext(
  'auth-failure',
  "You've logged in with an account number that is not valid. Please log out and try another one.",
);

const EXPIRED_ACCOUNT_MSG = pgettext(
  'auth-failure',
  'You have no more VPN time left on this account. Please log in on our website to buy more credit.',
);

const TOO_MANY_CONNECTIONS_MSG = pgettext(
  'auth-failure',
  'This account has too many simultaneous connections. Disconnect another device or try connecting again shortly.',
);

export class AuthFailure {
  private reasonId: AuthFailureKind;
  private message: string;

  constructor(reason?: string) {
    if (!reason) {
      log.error('Received invalid auth_failed reason: ', reason);
      this.reasonId = 'UNKNOWN';
      this.message = GENERIC_FAILURE_MSG;
      return;
    }

    const results = /^\[(\w+)\]\s*(.*)$/.exec(reason);

    if (!results || results.length < 3) {
      log.error(`Received invalid auth_failed message - "${reason}"`);
      this.reasonId = 'UNKNOWN';
      this.message = reason;
      return;
    }

    const idString = results[1];
    this.reasonId = strToFailureKind(idString);
    this.message = results[2] || GENERIC_FAILURE_MSG;
  }

  public show(): string {
    switch (this.reasonId) {
      case 'INVALID_ACCOUNT':
        return INVALID_ACCOUNT_MSG;
      case 'EXPIRED_ACCOUNT':
        return EXPIRED_ACCOUNT_MSG;
      case 'TOO_MANY_CONNECTIONS':
        return TOO_MANY_CONNECTIONS_MSG;
      case 'UNKNOWN':
        return this.message;
    }
  }
}

export function strToFailureKind(id: string): AuthFailureKind {
  switch (id) {
    case 'INVALID_ACCOUNT':
    case 'EXPIRED_ACCOUNT':
    case 'TOO_MANY_CONNECTIONS':
      return id;
    default:
      log.error(`Received unknown auth_failed message id - ${id}`);
      return 'UNKNOWN';
  }
}
