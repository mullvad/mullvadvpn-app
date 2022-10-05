import { expect } from 'chai';
import { it, describe } from 'mocha';
import * as date from '../../src/shared/date-helper';

describe('Date helper', () => {
  it('should modify minutes', () => {
    const initialDate = new Date('2021-01-01 13:37:10');
    const earlierDate = date.dateByAddingComponent(initialDate, date.DateComponent.minute, -50);
    const laterDate = date.dateByAddingComponent(initialDate, date.DateComponent.minute, 100);

    expect(earlierDate.getFullYear()).to.equal(2021);
    expect(earlierDate.getMonth()).to.equal(0);
    expect(earlierDate.getDate()).to.equal(1);
    expect(earlierDate.getHours()).to.equal(12);
    expect(earlierDate.getMinutes()).to.equal(47);
    expect(earlierDate.getSeconds()).to.equal(10);

    expect(laterDate.getFullYear()).to.equal(2021);
    expect(laterDate.getMonth()).to.equal(0);
    expect(laterDate.getDate()).to.equal(1);
    expect(laterDate.getHours()).to.equal(15);
    expect(laterDate.getMinutes()).to.equal(17);
    expect(laterDate.getSeconds()).to.equal(10);
  });

  it('should modify hours', () => {
    const initialDate = new Date('2021-01-01 13:37:10');
    const earlierDate = date.dateByAddingComponent(initialDate, date.DateComponent.hour, -50);
    const laterDate = date.dateByAddingComponent(initialDate, date.DateComponent.hour, 100);

    expect(earlierDate.getFullYear()).to.equal(2020);
    expect(earlierDate.getMonth()).to.equal(11);
    expect(earlierDate.getDate()).to.equal(30);
    expect(earlierDate.getHours()).to.equal(11);
    expect(earlierDate.getMinutes()).to.equal(37);
    expect(earlierDate.getSeconds()).to.equal(10);

    expect(laterDate.getFullYear()).to.equal(2021);
    expect(laterDate.getMonth()).to.equal(0);
    expect(laterDate.getDate()).to.equal(5);
    expect(laterDate.getHours()).to.equal(17);
    expect(laterDate.getMinutes()).to.equal(37);
    expect(laterDate.getSeconds()).to.equal(10);
  });

  it('should modify days', () => {
    const initialDate = new Date('2021-01-01 13:37:10');
    const earlierDate = date.dateByAddingComponent(initialDate, date.DateComponent.day, -50);
    const laterDate = date.dateByAddingComponent(initialDate, date.DateComponent.day, 100);

    expect(earlierDate.getFullYear()).to.equal(2020);
    expect(earlierDate.getMonth()).to.equal(10);
    expect(earlierDate.getDate()).to.equal(12);
    expect(earlierDate.getHours()).to.equal(13);
    expect(earlierDate.getMinutes()).to.equal(37);
    expect(earlierDate.getSeconds()).to.equal(10);

    expect(laterDate.getFullYear()).to.equal(2021);
    expect(laterDate.getMonth()).to.equal(3);
    expect(laterDate.getDate()).to.equal(11);
    expect(laterDate.getHours()).to.equal(13);
    expect(laterDate.getMinutes()).to.equal(37);
    expect(laterDate.getSeconds()).to.equal(10);
  });

  it('should calculate positive difference between dates', () => {
    const diff1 = new date.DateDiff('2021-01-14 13:37:10', '2021-02-01 14:40:12');
    expect(diff1.years).to.equal(0);
    expect(diff1.months).to.equal(0);
    expect(diff1.days).to.equal(18);
    expect(diff1.hours).to.equal(diff1.days * 24 + 1);
    expect(diff1.minutes).to.equal(diff1.hours * 60 + 3);
    expect(diff1.seconds).to.equal(diff1.minutes * 60 + 2);

    const diff2 = new date.DateDiff('2021-01-14 13:37:10', '2021-02-14 14:40:12');
    expect(diff2.years).to.equal(0);
    expect(diff2.months).to.equal(1);
    expect(diff2.days).to.equal(31);
    expect(diff2.hours).to.equal(diff2.days * 24 + 1);
    expect(diff2.minutes).to.equal(diff2.hours * 60 + 3);
    expect(diff2.seconds).to.equal(diff2.minutes * 60 + 2);

    const diff3 = new date.DateDiff('2021-01-14 13:37:10', '2022-01-14 13:37:09');
    expect(diff3.years).to.equal(0);
    expect(diff3.months).to.equal(11);
    expect(diff3.days).to.equal(364);
    expect(diff3.hours).to.equal(diff3.days * 24 + 23);
    expect(diff3.minutes).to.equal(diff3.hours * 60 + 59);
    expect(diff3.seconds).to.equal(diff3.minutes * 60 + 59);
  });

  it('should calculate negative difference between dates', () => {
    const diff1 = new date.DateDiff('2021-02-01 14:40:12', '2021-01-14 13:37:10');
    expect(diff1.years).to.equal(0);
    expect(diff1.months).to.equal(0);
    expect(diff1.days).to.equal(-18, 'aa');
    expect(diff1.hours).to.equal(diff1.days * 24 - 1);
    expect(diff1.minutes).to.equal(diff1.hours * 60 - 3);
    expect(diff1.seconds).to.equal(diff1.minutes * 60 - 2);
  });

  it('should format positive difference as string', () => {
    const diff1 = date.formatRelativeDate('2021-01-01 13:37:10', '2021-01-01 13:37:20');
    expect(diff1).to.equal('less than a day');

    const diff2 = date.formatRelativeDate('2021-01-01 13:37:10', '2021-01-02 13:37:20');
    expect(diff2).to.equal('1 day');

    const diff3 = date.formatRelativeDate('2021-01-01 13:37:10', '2021-02-25 13:37:20');
    expect(diff3).to.equal('55 days');

    const diff4 = date.formatRelativeDate('2021-01-01 13:37:10', '2021-04-25 13:37:20');
    expect(diff4).to.equal('3 months');

    const diff5 = date.formatRelativeDate('2021-01-01 13:37:10', '2031-04-25 13:37:20');
    expect(diff5).to.equal('10 years');
  });

  it('should format positive difference as string suffixed with "left"', () => {
    const diff1 = date.formatRelativeDate('2021-01-01 13:37:10', '2021-01-01 13:37:20', true);
    expect(diff1).to.equal('less than a day left');

    const diff2 = date.formatRelativeDate('2021-01-01 13:37:10', '2021-01-02 13:37:20', true);
    expect(diff2).to.equal('1 day left');

    const diff3 = date.formatRelativeDate('2021-01-01 13:37:10', '2021-02-25 13:37:20', true);
    expect(diff3).to.equal('55 days left');

    const diff4 = date.formatRelativeDate('2021-01-01 13:37:10', '2021-04-25 13:37:20', true);
    expect(diff4).to.equal('3 months left');

    const diff5 = date.formatRelativeDate('2021-01-01 13:37:10', '2031-04-25 13:37:20', true);
    expect(diff5).to.equal('10 years left');
  });

  it('should format negative difference as string', () => {
    const diff1 = date.formatRelativeDate('2021-01-01 13:37:20', '2021-01-01 13:37:10', true);
    expect(diff1).to.equal('less than a minute ago');

    const diff2 = date.formatRelativeDate('2021-01-02 13:37:20', '2021-01-01 13:37:10', true);
    expect(diff2).to.equal('a day ago');

    const diff3 = date.formatRelativeDate('2021-02-25 13:37:20', '2021-01-01 13:37:10', true);
    expect(diff3).to.equal('a month ago');

    const diff4 = date.formatRelativeDate('2021-04-25 13:37:20', '2021-01-01 13:37:10', true);
    expect(diff4).to.equal('3 months ago');

    const diff5 = date.formatRelativeDate('2031-04-25 13:37:20', '2021-01-01 13:37:10', true);
    expect(diff5).to.equal('10 years ago');
  });

  it('should format time left correctly', () => {
    expect(date.formatTimeLeft('2022-09-01', '2022-09-01')).to.equal('less than a day left');
    expect(date.formatTimeLeft('2022-09-01', '2022-09-02')).to.equal('1 day left');
    expect(date.formatTimeLeft('2022-09-01', '2022-09-05')).to.equal('4 days left');
    expect(date.formatTimeLeft('2022-09-01', '2022-09-30')).to.equal('29 days left');
    expect(date.formatTimeLeft('2022-09-01', '2023-09-01')).to.equal('365 days left');
    expect(date.formatTimeLeft('2022-09-01', '2024-08-30')).to.equal('729 days left');
    expect(date.formatTimeLeft('2022-09-01', '2024-08-31')).to.equal('2 years left');
    expect(date.formatTimeLeft('2022-09-01', '2024-09-05')).to.equal('2 years left');
    expect(date.formatTimeLeft('2022-09-01', '2025-08-31')).to.equal('2 years left');
    expect(date.formatTimeLeft('2022-09-01', '2025-09-01')).to.equal('3 years left');
  });
});
