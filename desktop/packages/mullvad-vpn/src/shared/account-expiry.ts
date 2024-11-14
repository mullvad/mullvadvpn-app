import {
  dateByAddingComponent,
  DateComponent,
  DateType,
  FormatDateOptions,
  formatRelativeDate,
} from './date-helper';

export function hasExpired(expiry: DateType): boolean {
  return new Date(expiry).getTime() < Date.now();
}

export function closeToExpiry(expiry: DateType, days = 3): boolean {
  return (
    !hasExpired(expiry) &&
    new Date(expiry) <= dateByAddingComponent(new Date(), DateComponent.day, days)
  );
}

export function formatDate(date: DateType, locale: string): string {
  return new Intl.DateTimeFormat(locale, { dateStyle: 'medium', timeStyle: 'short' }).format(
    new Date(date),
  );
}

export function formatRemainingTime(expiry: DateType, options?: FormatDateOptions): string {
  return formatRelativeDate(new Date(), expiry, options);
}
