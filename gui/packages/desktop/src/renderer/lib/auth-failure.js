// @flow

import log from 'electron-log';

export type AuthFailureKind =
  | 'INVALID_ACCOUNT'
  | 'EXPIRED_ACCOUNT'
  | 'TOO_MANY_CONNECTIONS'
  | 'UNKNOWN'
  | 'INVALID';

const GENERIC_FAILURE_MESSAGE = 'Account authentication failed';

export class AuthFailure {
  _reasonId: AuthFailureKind;
  _message: string;

  constructor(reason: ?string) {
    if (!reason) {
      log.error('failed to authenticate');
      this._reasonId = 'INVALID';
      this._message = GENERIC_FAILURE_MESSAGE;
      return;
    }

    const results = /^\[(\w+)\]\s*(.*?)$/.exec(reason);

    if (!results || results.length < 1) {
      log.error(`Received invalid auth_failed message - "${reason}"`);
      this._reasonId = 'INVALID';
      this._message = GENERIC_FAILURE_MESSAGE;
      return;
    }

    const id_string = results[1];
    this._reasonId = strToFailureKind(id_string);
    this._message = results[2] || GENERIC_FAILURE_MESSAGE;
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
        throw new Error(`Invalid reason ID: ${(this._reasonId: empty)}`);
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
