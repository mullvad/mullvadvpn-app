import { expect } from 'chai';
import { it, describe } from 'mocha';
import { parseAuthFailure, AuthFailureKind } from '../../src/shared/auth-failure';

describe('auth_failed parsing', () => {
  it('invalid line parsing works', () => {
    const authFailure = parseAuthFailure('invalid auth_failed message');
    expect(authFailure.kind).to.be.equal(AuthFailureKind.unknown);
    expect(authFailure.message).to.be.equal('invalid auth_failed message');
  });

  it('valid unknown works', () => {
    const authFailure = parseAuthFailure('[valid_unknown] Message');
    expect(authFailure.kind).to.be.equal(AuthFailureKind.unknown);
    expect(authFailure.message).to.be.equal('Message');
  });

  it('valid known works', () => {
    const authFailure = parseAuthFailure('[INVALID_ACCOUNT] Invalid account');
    expect(authFailure.kind).to.be.equal(AuthFailureKind.invalidAccount);
  });

  it('empty message works', () => {
    const authFailure = parseAuthFailure('[INVALID_ACCOUNT]');
    expect(authFailure.kind).to.be.equal(AuthFailureKind.invalidAccount);
  });
});
