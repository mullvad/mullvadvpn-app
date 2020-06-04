/* eslint-disable @typescript-eslint/no-explicit-any */

export type NotificationIndicatorType = 'success' | 'warning' | 'error';

export interface SystemNotification<TArguments extends any[]> {
  message: string | ((...args: TArguments) => string);
  important: boolean;
  supressInDevelopment?: boolean;
}

export interface InAppNotification<TArguments extends any[]> {
  indicator: NotificationIndicatorType;
  title: string;
  body?: string | ((...args: TArguments) => string);
}

export interface Notification<TArguments extends any[]> {
  condition: (...args: TArguments) => any;
  systemNotification?: SystemNotification<TArguments>;
  inAppNotification?: InAppNotification<TArguments>;
}

export function validateNotification<T extends any[], U extends Notification<T>>(
  notification: U,
): U {
  return notification;
}

export * from './accountExpiry';
export * from './blockWhenDisconnected';
export * from './connected';
export * from './connecting';
export * from './disconnected';
export * from './error';
export * from './inconsistentVersion';
export * from './nonBlockingError';
export * from './reconnecting';
export * from './unsupportedVersion';
export * from './updateAvailable';
