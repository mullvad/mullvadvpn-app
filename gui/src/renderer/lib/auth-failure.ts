import { messages } from '../../shared/gettext';

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
        "You've logged in with an account number that is not valid. Please log out and try another one.",
      );

    case AuthFailureKind.expiredAccount:
      return messages.pgettext(
        'auth-failure',
        'You have no more VPN time left on this account. Please log in on our website to buy more credit.',
      );

    case AuthFailureKind.tooManyConnections:
      return messages.pgettext(
        'auth-failure',
        'This account has too many simultaneous connections. Disconnect another device or try connecting again shortly.',
      );

    case AuthFailureKind.unknown:
      return messages.pgettext('auth-failure', 'Account authentication failed.');
  }
}
