import { expect } from 'chai';
import { it, describe } from 'mocha';
import { AuthFailureError, AuthFailureKind } from '../src/renderer/lib/auth-failure';

describe('auth_failed parsing', () => {
  it('invalid line parsing works', () => {
    const auth_msg = new AuthFailureError('invalid auth_failed message');
    expect(auth_msg.kind).to.be.equal(AuthFailureKind.unknown);
    expect(auth_msg.message).to.be.equal('invalid auth_failed message');
  });

  it('valid unknown works', () => {
    const auth_msg = new AuthFailureError('[valid_unknown] Message');
    expect(auth_msg.kind).to.be.equal(AuthFailureKind.unknown);
    expect(auth_msg.message).to.be.equal('Message');
  });

  it('valid known works', () => {
    const auth_msg = new AuthFailureError('[INVALID_ACCOUNT] Invalid account');
    expect(auth_msg.kind).to.be.equal(AuthFailureKind.invalidAccount);
  });

  it('empty message works', () => {
    const auth_msg = new AuthFailureError('[INVALID_ACCOUNT]');
    expect(auth_msg.kind).to.be.equal(AuthFailureKind.invalidAccount);
  });
});
