import { AuthFailure } from '../src/renderer/lib/auth-failure';
describe('auth_failed parsing', () => {
  it('invalid line parsing works', () => {
    const auth_msg = new AuthFailure('invalid auth_failed message');
    expect(auth_msg._reasonId).to.be.eql('INVALID');
    expect(auth_msg.show()).to.be.eql('Account authentication failed');
  });

  it('valid unknown works', () => {
    const auth_msg = new AuthFailure('[valid_unknown] Message');
    expect(auth_msg._reasonId).to.be.eql('UNKNOWN');
    expect(auth_msg.show()).to.be.eql('Message');
  });

  it('valid known works', () => {
    const auth_msg = new AuthFailure('[INVALID_ACCOUNT] Invalid account');
    expect(auth_msg._reasonId).to.be.eql('INVALID_ACCOUNT');
    expect(auth_msg.show()).to.be.eql('The account is invalid');
  });
});
