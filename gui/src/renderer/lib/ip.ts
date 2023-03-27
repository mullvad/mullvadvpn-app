// Number of groups for each IP format
type IPv4Octets = [number, number, number, number];
type IPv6Groups = [number, number, number, number, number, number, number, number];

// Number of bits in each group for each IP format
const IPv4OctetSize = 8;
const IPv6GroupSize = 16;

// Abstract class representing an IP address
export abstract class IpAddress<G extends number[]> {
  public constructor(public readonly groups: G) {}

  public abstract isLocal(): boolean;

  public static fromString(ip: string): IPv4Address | IPv6Address {
    try {
      return IPv4Address.fromString(ip);
    } catch (e) {
      return IPv6Address.fromString(ip);
    }
  }
}

// Abstract class representing an IP range or subnet
export abstract class IpRange<G extends number[]> {
  public constructor(public readonly groups: G, public readonly prefixSize: number) {}

  // Returns whether or not this subnet includes the provided IP
  protected includes<T extends IpAddress<G>>(ip: T, groupSize: number): boolean {
    return IpRange.match(groupSize, ip.groups, [this.groups, this.prefixSize]);
  }

  // Matches each group of the ip/subnet from left to right to determine if they match
  private static match(
    groupSize: number,
    [ipGroup, ...ipGroups]: number[],
    [[subnetGroup, ...subnetGroups], prefixSize]: [number[], number],
  ): boolean {
    if (prefixSize >= groupSize) {
      // If the current group is part of the prefix the only needed check is if they are equal
      return (
        ipGroup === subnetGroup &&
        IPv4Range.match(groupSize, ipGroups, [subnetGroups, prefixSize - groupSize])
      );
    } else {
      // If the group (or parts of the group) isn't part of the prefix the non-prefix part needs to
      // be compared
      // variableBits contains the maximum value that the non-prefix bits can have
      const variableBits = getBitsMax(groupSize - prefixSize);
      // Calculate smallest IP in the subnet
      const subnetMin = subnetGroup & (getBitsMax(groupSize) - variableBits);
      // Calculate greatest IP in the subnet
      const subnetMax = subnetGroup | variableBits;
      // Check if the provided ip is between subnetMin/-Max
      return ipGroup >= subnetMin && ipGroup <= subnetMax;
    }
  }
}

export class IPv4Address extends IpAddress<IPv4Octets> {
  public constructor(octets: IPv4Octets) {
    super(octets);

    // Ensure that each octets is the correct number of bits
    if (octets.some((octets) => !isNumberOfBits(octets, IPv4OctetSize))) {
      throw new Error(`Invalid ip: ${octets.join('.')}`);
    }
  }

  public isLocal(): boolean {
    const localSubnets = [...IPV4_LAN_SUBNETS, IPV4_LOOPBACK_SUBNET];
    return localSubnets.some((subnet) => subnet.includes(this));
  }

  // Parses an ip address from a string of the quad-dotted format, e.g. 127.0.0.1
  public static fromString(ip: string): IPv4Address {
    try {
      const octets = IPv4Address.octetsFromString(ip);
      return new IPv4Address(octets);
    } catch (e) {
      throw new Error(`Invalid ip: ${ip}`);
    }
  }

  public static octetsFromString(ip: string): IPv4Octets {
    try {
      const octets = ip.split('.');
      if (octets.every((octet) => /^\d{1,3}$/.test(octet))) {
        const parsedOctets = octets.map((octet) => parseInt(octet, 10));
        if (IPv4Address.isIPv4Octets(parsedOctets)) {
          return parsedOctets;
        }
      }
    } catch (e) {
      // no-op
    }

    throw new Error(`Invalid ip: ${ip}`);
  }

  public static isValid(ip: string): boolean {
    try {
      IPv4Address.fromString(ip);
      return true;
    } catch (e) {
      return false;
    }
  }

  // Makes sure that the number of octets is correct and values where parsed correctly
  private static isIPv4Octets(octets: number[]): octets is IPv4Octets {
    return octets.length === 4 && octets.every((octet) => !isNaN(octet));
  }
}

export class IPv4Range extends IpRange<IPv4Octets> {
  public constructor(octets: IPv4Octets, prefixSize: number) {
    super(octets, prefixSize);

    // Makes sure that the prefix is within the correct range
    if (prefixSize < 0 || prefixSize > 32) {
      throw new Error(`Invalid ip: ${octets.join('.')}/${prefixSize}`);
    }
  }

  public static fromString(subnet: string): IPv4Range {
    try {
      // In addition to parsing the ip the subnet-mask also needs to be parsed
      const parts = subnet.split('/');
      if (/^\d{1,2}$/.test(parts[1])) {
        const octets = IPv4Address.octetsFromString(parts[0]);
        const prefixSize = parseInt(parts[1]);
        return new IPv4Range(octets, prefixSize);
      }
    } catch (e) {
      // no-op
    }

    throw new Error(`Invalid ip: ${subnet}`);
  }

  public includes(ip: IPv4Address): boolean {
    return super.includes(ip, IPv4OctetSize);
  }
}

export class IPv6Address extends IpAddress<IPv6Groups> {
  public constructor(groups: IPv6Groups) {
    super(groups);

    // Ensure that each group is the correct number of bits
    if (groups.some((group) => !isNumberOfBits(group, 16))) {
      throw new Error(`Invalid ip: ${groups.join(':')}`);
    }
  }

  public isLocal(): boolean {
    const localSubnets = [...IPV6_LAN_SUBNETS, IPV6_LOOPBACK_SUBNET];
    return localSubnets.some((subnet) => subnet.includes(this));
  }

  // Parses IPv6 addresses where the groups are separated by ':' and supports shortened addresses.
  public static fromString(ip: string): IPv6Address {
    try {
      const groups = IPv6Address.groupsFromString(ip);
      return new IPv6Address(groups);
    } catch (e) {
      throw new Error(`Invalid ip: ${ip}`);
    }
  }

  public static groupsFromString(ip: string): IPv6Groups {
    try {
      // Split on shortening separator and make sure there's only one separator
      const shortened = ip.split('::');
      if (shortened.length <= 2) {
        // Split each part of the shortened address into groups and remove any empty groups, such as
        // the one before the separator in ::1
        const parts = shortened.map((groups) => groups.split(':').filter((group) => group !== ''));

        let groups: string[];
        if (parts.length === 2) {
          // If the address contained the shortening separator the parts are concatenated with empty
          // groups in between
          const shortened = Array(8 - parts[0].length - parts[1].length).fill(0x0);
          groups = [...parts[0], ...shortened, ...parts[1]];
        } else {
          // If it wasn't shortened all groups are used as is
          groups = parts.flat();
        }

        if (groups.every((group) => /^[0-9a-fA-F]{1,4}$/.test(group))) {
          const parsedGroups = groups.map((group) => parseInt(group, 16));

          if (IPv6Address.isIPv6Groups(parsedGroups)) {
            return parsedGroups;
          }
        }
      }
    } catch (e) {
      // no-op
    }

    throw new Error(`Invalid ip: ${ip}`);
  }

  public static isValid(ip: string): boolean {
    try {
      IPv6Address.fromString(ip);
      return true;
    } catch (e) {
      return false;
    }
  }

  // Makes sure that the number of groups is correct and values where parsed correctly
  private static isIPv6Groups(groups: number[]): groups is IPv6Groups {
    return groups.length === 8 && groups.every((group) => !isNaN(group));
  }
}

export class IPv6Range extends IpRange<IPv6Groups> {
  public constructor(groups: IPv6Groups, prefixSize: number) {
    super(groups, prefixSize);

    // Makes sure that the prefix is within the correct range
    if (prefixSize < 0 || prefixSize > 128) {
      throw new Error(`Invalid subnet: ${groups.join(':')}/${prefixSize}`);
    }
  }

  public static fromString(subnet: string): IPv6Range {
    try {
      // In addition to parsing the ip the subnet-mask also needs to be parsed
      const parts = subnet.split('/');
      if (/^\d{1,3}$/.test(parts[1])) {
        const groups = IPv6Address.groupsFromString(parts[0]);
        const prefixSize = parseInt(parts[1], 10);
        return new IPv6Range(groups, prefixSize);
      }
    } catch (e) {
      // no-op
    }

    throw new Error(`Invalid subnet: ${subnet}`);
  }

  public includes(ip: IPv6Address): boolean {
    return super.includes(ip, IPv6GroupSize);
  }
}

// Returns the maximum value possible with the provided size
function getBitsMax(bits: number): number {
  return Math.pow(2, bits) - 1;
}

// Returns whether or not a number is possible to represent as an unsigned in of the provided size
function isNumberOfBits(value: number, bits: number): boolean {
  return value >= 0 && value < Math.pow(2, bits);
}

// IPv4 addresses reserved for local networks
const IPV4_LAN_SUBNETS = [
  new IPv4Range([10, 0, 0, 0], 8),
  new IPv4Range([172, 16, 0, 0], 12),
  new IPv4Range([192, 168, 0, 0], 16),
  new IPv4Range([169, 254, 0, 0], 16),
];

// IPv6 addresses reserved for local networks
const IPV6_LAN_SUBNETS = [
  new IPv6Range([0xfe80, 0, 0, 0, 0, 0, 0, 0], 10),
  new IPv6Range([0xfc00, 0, 0, 0, 0, 0, 0, 0], 7),
];

const IPV4_LOOPBACK_SUBNET = new IPv4Range([127, 0, 0, 0], 8);
const IPV6_LOOPBACK_SUBNET = new IPv6Range([0, 0, 0, 0, 0, 0, 0, 1], 128);
