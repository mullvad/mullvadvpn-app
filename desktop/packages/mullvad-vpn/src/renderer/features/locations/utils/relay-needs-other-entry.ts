import type { RelaySelectorRelayMatch } from '../../../../shared/relay-selector-rpc-types';

export function relayNeedsOtherEntry(hostname: string, matches: RelaySelectorRelayMatch[]) {
  const match = matches.find((match) => match.relay.hostname === hostname);
  return match?.metadata.needsOtherEntry ?? false;
}
