export interface ConnectedRelay {
  hostname: string;
  inIp: string;
  inPort: number;
  inProtocol: string;
  obfuscationType?: string;
  outIpv4?: string;
}

export function formatInAddress(relay: ConnectedRelay) {
  // The app shows IPv6 addresses in brackets
  const inIp = relay.inIp.includes(':') ? `[${relay.inIp}]` : relay.inIp;
  return `${inIp}:${relay.inPort} ${relay.inProtocol.toUpperCase()}`;
}
