import log from 'electron-log';
import { pgettext } from '../../shared/gettext';

export enum AuthFailureKind {
  invalidAccount,
  expiredAccount,
  tooManyConnections,
  unknown,
}

export class AuthFailureError extends Error {
  private kindValue: AuthFailureKind;
  private unknownErrorMessage?: string;

  get kind(): AuthFailureKind {
    return this.kindValue;
  }

  get message(): string {
    switch (this.kindValue) {
      case AuthFailureKind.invalidAccount:
        return pgettext(
          'auth-failure',
          "You've logged in with an account number that is not valid. Please log out and try another one.",
        );

      case AuthFailureKind.expiredAccount:
        return pgettext(
          'auth-failure',
          'You have no more VPN time left on this account. Please log in on our website to buy more credit.',
        );

      case AuthFailureKind.tooManyConnections:
        return pgettext(
          'auth-failure',
          'This account has too many simultaneous connections. Disconnect another device or try connecting again shortly.',
        );

      case AuthFailureKind.unknown:
        return (
          this.unknownErrorMessage || pgettext('auth-failure', 'Account authentication failed.')
        );
    }
  }

  constructor(reason?: string) {
    super();

    if (!reason) {
      log.error('Received invalid auth_failed reason: ', reason);

      this.kindValue = AuthFailureKind.unknown;
      return;
    }

    const results = /^\[(\w+)\]\s*(.*)$/.exec(reason);

    if (results && results.length === 3) {
      const rawReasonId = results[1];
      const kindValue = rawReasonIdToFailureKind(rawReasonId);

      if (kindValue === AuthFailureKind.unknown) {
        log.error(`Received unknown auth_failed message id - ${rawReasonId}`);
      }

      this.kindValue = kindValue;
      this.unknownErrorMessage = results[2];
    } else {
      log.error(`Received invalid auth_failed message - "${reason}"`);

      this.kindValue = AuthFailureKind.unknown;
      this.unknownErrorMessage = reason;
    }
  }
}

function rawReasonIdToFailureKind(id: string): AuthFailureKind {
  // These strings should match up with mullvad-types/src/auth_failed.rs
  switch (id) {
    case 'INVALID_ACCOUNT':
      return AuthFailureKind.invalidAccount;

    case 'EXPIRED_ACCOUNT':
      return AuthFailureKind.expiredAccount;

    case 'TOO_MANY_CONNECTIONS':
      return AuthFailureKind.tooManyConnections;

    default:
      return AuthFailureKind.unknown;
  }
}
