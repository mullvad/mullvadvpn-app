export class NoCreditError extends Error {
  constructor() {
    super("Account doesn't have enough credit available for connection");
  }

  get userFriendlyTitle(): string {
    return 'Out of time';
  }

  get userFriendlyMessage(): string {
    return 'Buy more time, so you can continue using the internet securely';
  }
}

export class NoInternetError extends Error {
  constructor() {
    super('Internet connectivity is currently unavailable');
  }

  get userFriendlyTitle(): string {
    return 'Offline';
  }

  get userFriendlyMessage(): string {
    return 'Your internet connection will be secured when you get back online';
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

export class UnknownError extends Error {
  constructor(cause: string) {
    super(`An unknown error occurred, ${cause}`);
  }

  get userFriendlyTitle(): string {
    return 'Something went wrong';
  }
}
