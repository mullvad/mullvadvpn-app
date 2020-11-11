import { messages } from './gettext';

export enum AuthFailureKind {
  invalidAccount,
  expiredAccount,
  tooManyConnections,
  unknown,
}

interface IAuthFailure {
  kind: AuthFailureKind;
  message: string;
}

export function parseAuthFailure(rawFailureMessage?: string): IAuthFailure {
  if (rawFailureMessage) {
    const results = /^\[(\w+)\]\s*(.*)$/.exec(rawFailureMessage);

    if (results && results.length === 3) {
      const kind = parseRawFailureKind(results[1]);
      const message = kind === AuthFailureKind.unknown ? results[2] : messageForFailureKind(kind);

      return {
        kind,
        message,
      };
    } else {
      return {
        kind: AuthFailureKind.unknown,
        message: rawFailureMessage,
      };
    }
  } else {
    return {
      kind: AuthFailureKind.unknown,
      message: messageForFailureKind(AuthFailureKind.unknown),
    };
  }
}

function parseRawFailureKind(failureId: string): AuthFailureKind {
  // These strings should match up with mullvad-types/src/auth_failed.rs
  switch (failureId) {
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

function messageForFailureKind(kind: AuthFailureKind): string {
  switch (kind) {
    case AuthFailureKind.invalidAccount:
      return messages.pgettext(
        'auth-failure',
        'You are logged in with an invalid account number. Please log out and try another one.',
      );

    case AuthFailureKind.expiredAccount:
      return messages.pgettext('auth-failure', 'Blocking internet: account is out of time');

    case AuthFailureKind.tooManyConnections:
      return messages.pgettext(
        'auth-failure',
        'Too many simultaneous connections on this account. Disconnect another device or try connecting again shortly.',
      );

    case AuthFailureKind.unknown:
      return messages.pgettext(
        'auth-failure',
        'Unable to authenticate account. Please contact support.',
      );
  }
}
