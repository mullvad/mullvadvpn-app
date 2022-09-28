import { dateByAddingComponent, DateComponent, DateType, formatTimeLeft } from './date-helper';
import { capitalize } from './string-helpers';

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

export function formatRemainingTime(
  expiry: DateType,
  shouldCapitalizeFirstLetter?: boolean,
): string {
  const remaining = formatTimeLeft(new Date(), expiry);
  return shouldCapitalizeFirstLetter ? capitalize(remaining) : remaining;
}
