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

export class TooManyDevicesError extends Error {
  constructor() {
    super('Too many devices');
  }
}

export class ListDevicesError extends Error {
  constructor() {
    super('Failed to fetch list of devices');
  }
}
