export class NoCreditError extends Error {
  constructor() {
    super("Account doesn't have enough credit available for connection");
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

export class CommunicationError extends Error {
  constructor() {
    super('api.mullvad.net is blocked, please check your firewall');
  }
}
