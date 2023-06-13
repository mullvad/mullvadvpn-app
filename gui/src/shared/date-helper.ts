import { sprintf } from 'sprintf-js';

import { messages } from './gettext';
import { capitalize } from './string-helpers';

export type DateType = Date | string | number;

export enum DateComponent {
  day,
  hour,
  minute,
}

export function dateByAddingComponent(date: DateType, component: DateComponent, value: number) {
  const modifiedDate = new Date(date);
  switch (component) {
    case DateComponent.day:
      modifiedDate.setDate(modifiedDate.getDate() + value);
      break;
    case DateComponent.hour:
      modifiedDate.setHours(modifiedDate.getHours() + value);
      break;
    case DateComponent.minute:
      modifiedDate.setMinutes(modifiedDate.getMinutes() + value);
      break;
  }

  return modifiedDate;
}

export class DateDiff {
  private readonly fromDate: Date;
  private readonly toDate: Date;

  public constructor(fromDate: DateType, toDate: DateType) {
    this.fromDate = new Date(fromDate);
    this.toDate = new Date(toDate);
  }

  get milliseconds(): number {
    return this.toDate.getTime() - this.fromDate.getTime();
  }

  get seconds(): number {
    return this.floor(this.milliseconds / 1000);
  }

  get minutes(): number {
    return this.floor(this.seconds / 60);
  }

  get hours(): number {
    return this.floor(this.minutes / 60);
  }

  get days(): number {
    return this.floor(this.hours / 24);
  }

  get months(): number {
    const months = new Date(Math.abs(this.milliseconds)).getUTCMonth();
    const monthsWithSign = this.milliseconds >= 0 ? months : -months;
    return this.years * 12 + monthsWithSign;
  }

  get years(): number {
    const years = new Date(Math.abs(this.milliseconds)).getUTCFullYear() - 1970;
    return this.milliseconds >= 0 ? years : -years;
  }

  private floor(n: number): number {
    return n >= 0 ? Math.floor(n) : Math.ceil(n);
  }
}

export interface FormatDateOptions {
  suffix?: boolean;
  displayMonths?: boolean;
  capitalize?: boolean;
}

// If withSuffix is true then "left" will be added at the end of the remaining time.
// If noMonths is true then the following applies:
//  If a user has more than 2 years (730 days) left of time it should be displayed in whole years
//  rounded down If a user has less than 2 years left (e.g. 729 days) then this should be displayed
//  in days.
export function formatRelativeDate(
  fromDate: DateType,
  toDate: DateType,
  options?: FormatDateOptions,
): string {
  const diff = new DateDiff(fromDate, toDate);
  const years = Math.abs(diff.years);
  const months = Math.abs(diff.months);
  const days = Math.abs(diff.days);

  if (isNaN(years) || isNaN(months) || isNaN(days)) {
    return '';
  }

  let result = '';
  if (!options?.suffix) {
    if (options?.displayMonths ? years > 0 : days >= 730) {
      result = sprintf(messages.ngettext('1 year', '%d years', years), years);
    } else if (options?.displayMonths && months >= 3) {
      result = sprintf(messages.ngettext('1 month', '%d months', months), months);
    } else if (days > 0) {
      result = sprintf(messages.ngettext('1 day', '%d days', days), days);
    } else {
      result = messages.gettext('less than a day');
    }
  } else if (diff.milliseconds > 0) {
    if (options?.displayMonths ? years > 0 : days >= 730) {
      result = sprintf(messages.ngettext('1 year left', '%d years left', years), years);
    } else if (options?.displayMonths && months >= 3) {
      result = sprintf(messages.ngettext('1 month left', '%d months left', months), months);
    } else if (days > 0) {
      result = sprintf(messages.ngettext('1 day left', '%d days left', days), days);
    } else {
      result = messages.gettext('less than a day left');
    }
  }

  return options?.capitalize ? capitalize(result) : result;
}
