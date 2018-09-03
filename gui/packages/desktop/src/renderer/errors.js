// @flow

import type { BlockReason } from './lib/daemon-rpc';

export class BlockedError extends Error {
  constructor(reason: BlockReason) {
    switch (reason) {
      case 'auth_failed':
        super('Authentication failed');
        break;
      case 'set_security_policy_error':
        super('Failed to apply security policy');
        break;
      case 'start_tunnel_error':
        super('Failed to start tunnel connection');
        break;
      case 'no_matching_relay':
        super('No relay server matches the current settings');
        break;
      case 'no_account_token':
        super('No account token configured');
        break;
      default:
        super(`Unknown error: ${(reason: empty)}`);
    }
  }
}

export class NoCreditError extends Error {
  constructor() {
    super("Account doesn't have enough credit available for connection");
  }
}

export class NoInternetError extends Error {
  constructor() {
    super('Internet connectivity is currently unavailable');
  }
}

export class NoDaemonError extends Error {
  constructor() {
    super('Could not connect to Mullvad daemon');
  }
}

export class InvalidAccountError extends Error {
  constructor() {
    super('Invalid account number');
  }
}

export class NoAccountError extends Error {
  constructor() {
    super('No account was set');
  }
}

export class CommunicationError extends Error {
  constructor() {
    super('api.mullvad.net is blocked, please check your firewall');
  }
}
