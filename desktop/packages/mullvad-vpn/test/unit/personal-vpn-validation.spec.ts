import { describe, expect, it } from 'vitest';

import {
  validateAllowedIp,
  validateEndpoint,
  validateIp,
  validateWireguardKey,
} from '../../src/renderer/features/personal-vpn/lib/validate';

const VALID_KEY = Buffer.alloc(32, 1).toString('base64');

describe('validateWireguardKey', () => {
  it('accepts a canonical base64 32-byte key', () => {
    expect(validateWireguardKey(VALID_KEY)).to.be.undefined;
  });

  it('rejects empty', () => {
    expect(validateWireguardKey('')).to.equal('empty');
  });

  it('rejects a key of the wrong byte length', () => {
    const short = Buffer.alloc(16, 1).toString('base64');
    expect(validateWireguardKey(short)).to.equal('invalid');
  });

  it('rejects non-base64 garbage', () => {
    expect(validateWireguardKey('not a key!')).to.equal('invalid');
  });
});

describe('validateIp', () => {
  it('accepts IPv4', () => {
    expect(validateIp('10.0.0.1')).to.be.undefined;
    expect(validateIp('255.255.255.255')).to.be.undefined;
  });

  it('rejects out-of-range IPv4 octets', () => {
    expect(validateIp('256.0.0.1')).to.equal('invalid');
  });

  it('accepts IPv6', () => {
    expect(validateIp('fe80::1')).to.be.undefined;
    expect(validateIp('2001:db8::1')).to.be.undefined;
  });

  it('rejects hostnames', () => {
    expect(validateIp('example.com')).to.equal('invalid');
  });

  it('rejects empty', () => {
    expect(validateIp('')).to.equal('empty');
  });
});

describe('validateEndpoint', () => {
  it('accepts IPv4 host:port', () => {
    expect(validateEndpoint('10.0.0.1:51820')).to.be.undefined;
  });

  it('accepts bracketed IPv6 host:port', () => {
    expect(validateEndpoint('[fe80::1]:51820')).to.be.undefined;
  });

  it('rejects missing port', () => {
    expect(validateEndpoint('10.0.0.1')).to.equal('invalid-address');
  });

  it('rejects out-of-range port', () => {
    expect(validateEndpoint('10.0.0.1:70000')).to.equal('invalid-port');
  });

  it('rejects empty', () => {
    expect(validateEndpoint('')).to.equal('empty');
  });

  it('rejects hostname host', () => {
    expect(validateEndpoint('example.com:51820')).to.equal('invalid-address');
  });
});

describe('validateAllowedIp', () => {
  it('accepts non-blank values', () => {
    expect(validateAllowedIp('0.0.0.0/0')).to.be.undefined;
  });

  it('rejects blank', () => {
    expect(validateAllowedIp('   ')).to.equal('empty');
  });
});
