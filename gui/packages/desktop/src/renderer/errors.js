// @flow

import type { BlockReason } from './lib/daemon-rpc';

export class BlockedError extends Error {
  constructor(reason: BlockReason) {
    const message = (function() {
      switch (reason) {
        case 'enable_ipv6_error':
          return 'Could not configure IPv6, please enable it on your system or disable it in the app';
        case 'set_security_policy_error':
          return 'Failed to apply security policy';
        case 'start_tunnel_error':
          return 'Failed to start tunnel connection';
        case 'no_matching_relay':
          return 'No relay server matches the current settings';
        case 'no_account_token':
          return 'No account token configured';
        default:
          return `Unknown error: ${(reason: empty)}`;
      }
    })();

    super(message);
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
