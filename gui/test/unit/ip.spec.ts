import { expect } from 'chai';
import { it, describe } from 'mocha';
import * as ip from '../../src/renderer/lib/ip';

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

const validIpv4Subnets = ['10.0.0.0/0', '10.0.0.0/8', '10.0.0.0/32'];
const invalidIpv4Subnets = ['10.0.0.0', '10.0.0.0/', '10.0.0.0/-1', '10.0.0.0/33'];

const validIpv6Subnets = ['::1/0', 'fe::/128', '1:1::1/12', '0:1:2:3:4:5:6:7/64'];
const invalidIpv6Subnets = ['::1', 'fe::/', '0:0:0:0:0:0:0:0/-1', '::1/129'];

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

describe('IP', () => {
  it('should detect that valid IPv4 addresses are valid', () => {
    validIpv4Addresses.forEach((ipAddress) => {
      const valid = ip.IPv4Address.isValid(ipAddress);
      expect(valid).to.be.true;
      expect(() => ip.IPv4Address.fromString(ipAddress)).to.not.throw();
    });
  });

  it('should detect that invalid IPv4 addresses are invalid', () => {
    invalidIpv4Addresses.forEach((ipAddress) => {
      const valid = ip.IPv4Address.isValid(ipAddress);
      expect(valid).to.be.false;
      expect(() => ip.IPv4Address.fromString(ipAddress)).to.throw();
    });
  });

  it('should detect that valid IPv6 addresses are valid', () => {
    validIpv6Addresses.forEach((ipAddress) => {
      const valid = ip.IPv6Address.isValid(ipAddress);
      expect(valid).to.be.true;
      expect(() => ip.IPv6Address.fromString(ipAddress)).to.not.throw();
    });
  });

  it('should detect that invalid IPv6 addresses are invalid', () => {
    invalidIpv6Addresses.forEach((ipAddress) => {
      const valid = ip.IPv6Address.isValid(ipAddress);
      expect(valid).to.be.false;
      expect(() => ip.IPv6Address.fromString(ipAddress)).to.throw();
    });
  });

  it('should detect that valid IPv4 subnets are valid', () => {
    validIpv4Subnets.forEach((subnet) => {
      expect(() => ip.IPv4Range.fromString(subnet)).to.not.throw();
    });
  });

  it('should detect that invalid IPv4 subnets are invalid', () => {
    invalidIpv4Subnets.forEach((subnet) => {
      expect(() => ip.IPv4Range.fromString(subnet)).to.throw();
    });
  });

  it('should detect that valid IPv6 subnets are valid', () => {
    validIpv6Subnets.forEach((subnet) => {
      expect(() => ip.IPv6Range.fromString(subnet)).to.not.throw();
    });
  });

  it('should detect that invalid IPv6 subnets are invalid', () => {
    invalidIpv6Subnets.forEach((subnet) => {
      expect(() => ip.IPv6Range.fromString(subnet)).to.throw();
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

  it('should correctly parse IP addresses', () => {
    expect(ip.IpAddress.fromString('127.0.0.1').groups).to.deep.equal([127, 0, 0, 1]);
    expect(ip.IpAddress.fromString('1.1.1.1').groups).to.deep.equal([1, 1, 1, 1]);
    expect(ip.IpAddress.fromString('252.253.254.255').groups).to.deep.equal([252, 253, 254, 255]);

    const ip1 = ip.IpAddress.fromString('0:1:2:3:4:5:6:7').groups;
    expect(ip1).to.deep.equal([0, 1, 2, 3, 4, 5, 6, 7]);

    const ip2 = ip.IpAddress.fromString('ffff::').groups;
    expect(ip2).to.deep.equal([0xffff, 0, 0, 0, 0, 0, 0, 0]);

    const ip3 = ip.IpAddress.fromString('::1').groups;
    expect(ip3).to.deep.equal([0, 0, 0, 0, 0, 0, 0, 1]);

    const ip4 = ip.IpAddress.fromString('ffff:1::1').groups;
    expect(ip4).to.deep.equal([0xffff, 1, 0, 0, 0, 0, 0, 1]);
  });

  it('should correctly parse IP range prefix sizes', () => {
    expect(ip.IPv4Range.fromString('127.0.0.1/0').prefixSize).to.equal(0);
    expect(ip.IPv4Range.fromString('1.1.1.1/32').prefixSize).to.equal(32);

    expect(ip.IPv6Range.fromString('0:1:2:3:4:5:6:7/0').prefixSize).to.equal(0);
    expect(ip.IPv6Range.fromString('ffff::/128').prefixSize).to.equal(128);
    expect(ip.IPv6Range.fromString('::1/32').prefixSize).to.equal(32);
  });
});
