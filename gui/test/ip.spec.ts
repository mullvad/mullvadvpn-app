import { expect } from 'chai';
import { it, describe } from 'mocha';
import * as ip from '../src/renderer/lib/ip';

const validIpv4Addresses = [
  '127.0.0.1',
  '10.255.255.255',
  '192.168.1.1',
  '192.168.0.10',
  '192.168.1.254',
  '192.168.254.254',
  '10.0.0.1',
  '10.90.90.90',
  '1.1.1.1',
  '193.138.218.74',
];

const validIpv6Addresses = [
  '0:1:2:3:4:5:6:7',
  '00:11:22:33:44:55:66:77',
  '000:111:222:333:444:555:666:777',
  '0000:1111:2222:3333:4444:5555:6666:7777',
  'ffff::',
  '::ff:2233',
  'fee::ff:2233',
];

const invalidIpv4Addresses = [
  '127.0.0.0.1',
  '10.0.0.256',
  '192.168.1',
  '0.0.0.a1',
  '0.0.0.',
  '0.0..0',
  '0. 0.0.0',
  '0.a.0.0.0',
];

const invalidIpv6Addresses = [
  '00:11:22:33:44:55:66:77:88',
  '00:11:22:33:44:55:66',
  'ff::ff::ff',
  'ff::ff::',
  '::ff::',
  '00:11:22:33:44:55:66:gg',
  '13245:11:22:33:44:55:66:77',
  '::1g',
  'gg:11:22:33:44:55:66:77:88',
];

const localIpAddresses = [
  '10.0.0.0',
  '10.255.255.255',
  '172.16.0.0',
  '172.31.255.255',
  '192.168.0.0',
  '192.168.255.255',
];

const publicIpAddresses = [
  '1.1.1.1',
  '193.138.218.74',
  '9.255.255.255',
  '11.0.0.0',
  '172.15.0.0',
  '172.15.255.255',
  '172.32.0.0',
  '192.167.0.0',
  '192.167.255.255',
  '192.169.0.0',
];

describe('Logging', () => {
  it('should detect that valid IPv4 addresses are valid', () => {
    validIpv4Addresses.forEach((ipAddress) => {
      const valid = ip.IPv4Address.isValid(ipAddress);
      expect(valid).to.be.true;
      expect(() => ip.IPv4Address.fromString(ipAddress)).to.throw;
    });
  });

  it('should detect that invalid IPv4 addresses are invalid', () => {
    invalidIpv4Addresses.forEach((ipAddress) => {
      const valid = ip.IPv4Address.isValid(ipAddress);
      expect(valid).to.be.false;
      expect(() => ip.IPv4Address.fromString(ipAddress)).to.not.throw;
    });
  });

  it('should detect that valid IPv6 addresses are valid', () => {
    validIpv6Addresses.forEach((ipAddress) => {
      const valid = ip.IPv6Address.isValid(ipAddress);
      expect(valid).to.be.true;
      expect(() => ip.IPv6Address.fromString(ipAddress)).to.throw;
    });
  });

  it('should detect that invalid IPv6 addresses are invalid', () => {
    invalidIpv6Addresses.forEach((ipAddress) => {
      const valid = ip.IPv6Address.isValid(ipAddress);
      expect(valid).to.be.false;
      expect(() => ip.IPv6Address.fromString(ipAddress)).to.not.throw;
    });
  });

  it('should detect that IP addresses are local', () => {
    localIpAddresses.forEach((ipAddress) => {
      expect(ip.IpAddress.fromString(ipAddress).isLocal()).to.be.true;
    });
  });

  it('should detect that IP addresses are public', () => {
    publicIpAddresses.forEach((ipAddress) => {
      expect(ip.IpAddress.fromString(ipAddress).isLocal()).to.be.false;
    });
  });
});
