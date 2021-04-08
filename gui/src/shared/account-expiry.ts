import { DateComponent, DateType, formatRelativeDate, dateByAddingComponent } from './date-helper';
import { capitalize } from './string-helpers';

export function hasExpired(expiry: DateType): boolean {
  return new Date(expiry).getTime() < Date.now();
}

export function closeToExpiry(expiry: DateType): boolean {
  return (
    !hasExpired(expiry) &&
    new Date(expiry) <= dateByAddingComponent(new Date(), DateComponent.day, 3)
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
  const remaining = formatRelativeDate(new Date(), expiry, true);
  return shouldCapitalizeFirstLetter ? capitalize(remaining) : remaining;
}
