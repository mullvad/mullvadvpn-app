// @flow

import log from 'electron-log';

export type AuthFailureKind =
  | 'INVALID_ACCOUNT'
  | 'EXPIRED_ACCOUNT'
  | 'TOO_MANY_CONNECTIONS'
  | 'UNKNOWN'
  | 'INVALID';

export class AuthFailure {
  _reasonId: AuthFailureKind;
  _message: string;

  constructor(reason: ?string) {
    if (!reason) {
      log.error('failed to authenticate');
      this._reasonId = 'INVALID';
      this._message = 'Account authentication failed';
      return;
    }

    const bracket_idx = reason.indexOf(']');
    if (reason[0] != '[' || reason.length < 3 || bracket_idx === -1) {
      log.error(`Received invalid auth_failed message - "${reason}"`);
      this._reasonId = 'INVALID';
      this._message = 'Account authentication failed';
      return;
    }

    const id_str = reason.substring(1, bracket_idx);
    this._reasonId = strToFailureKind(id_str);
    this._message = reason.substring(bracket_idx + 1).trim();
  }

  show(): string {
    switch (this._reasonId) {
      case 'INVALID_ACCOUNT':
        return 'The account is invalid';
      case 'EXPIRED_ACCOUNT':
        return 'The account has expired';
      case 'TOO_MANY_CONNECTIONS':
        return 'The account has too many active connections';
      case 'INVALID':
      case 'UNKNOWN':
        return this._message;

      default:
        throw new Error(`Invalid reason ID: ${this._reasonId}`);
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
