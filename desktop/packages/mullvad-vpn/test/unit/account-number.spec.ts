import { describe, expect, it } from 'vitest';

import { isAccountNumber } from '../../src/shared/utils';

describe('Account number validation', () => {
  it('Should identify valid account numbers', () => {
    // Allowed account numbers can range between 10 and 16 digits
    expect(isAccountNumber('1234567890123456')).to.be.true;
    expect(isAccountNumber('123456789012')).to.be.true;
    expect(isAccountNumber('1234567890')).to.be.true;
  });

  it('Should reject too long account number', () => {
    expect(isAccountNumber('12345678901234567')).to.be.false;
  });

  it('Should reject non-digit account numbers', () => {
    expect(isAccountNumber('123456789012345a')).to.be.false;
    expect(isAccountNumber('12345678901e')).to.be.false;
    expect(isAccountNumber('123456789a')).to.be.false;
  });

  it('Should reject too short account number', () => {
    expect(isAccountNumber('123456789')).to.be.false;
  });
});
