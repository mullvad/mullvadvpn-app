import { expect } from '@playwright/test';
import { exec as execAsync } from 'child_process';
import { Page } from 'playwright';
import { promisify } from 'util';

import { ConnectedRelay } from '../../lib/tunnel-helpers';
import { expectConnected, expectDisconnected } from '../../shared/tunnel-state';

const exec = promisify(execAsync);

// Disconnects, applies `changeSettings` and connects again.
export async function reconnectWith(page: Page, changeSettings: () => Promise<unknown>) {
  // Disconnect first so `expectConnected` does not resolve for the old tunnel.
  await exec('mullvad disconnect --wait');
  await expectDisconnected(page);
  await changeSettings();
  await exec('mullvad connect --wait');
  await expectConnected(page);
}

// Returns the relay that the daemon is connected to
export async function getConnectedRelay(): Promise<ConnectedRelay> {
  const { stdout } = await exec('mullvad status --json');
  const tunnelState = JSON.parse(stdout);
  expect(tunnelState.state).toBe('connected');

  const { endpoint, location } = tunnelState.details;
  // The in address is that of the obfuscation endpoint if obfuscation is used, otherwise that of
  // the entry relay if multihop is used. This mirrors what the connection panel shows.
  const obfuscation = endpoint.obfuscation?.Single;
  const inEndpoint = obfuscation?.endpoint ?? endpoint.entry_endpoint ?? endpoint;
  const [, inIp, inPort] = /^\[?(.*?)\]?:(\d+)$/.exec(inEndpoint.address)!;

  return {
    hostname: location.hostname,
    inIp,
    inPort: Number(inPort),
    inProtocol: inEndpoint.protocol,
    obfuscationType: obfuscation?.obfuscation_type.toLowerCase(),
    outIpv4: location.ipv4 ?? undefined,
  };
}

// Returns the public IPv4 address of the tunnel. The daemon looks this up after connecting, so this
// waits for at most the `expect` timeout for it to become known.
export async function getOutIpv4(): Promise<string> {
  let outIpv4: string | undefined;
  await expect
    .poll(async () => {
      outIpv4 = (await getConnectedRelay()).outIpv4;
      return outIpv4;
    })
    .toBeDefined();
  return outIpv4!;
}
