import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../shared/gettext';
import { useRelayLocations } from '../../../../../../features/locations/hooks';
import { findCountryForRelay, findRelay } from '../../../../../../features/locations/utils';
import { useConnection } from '../../../../../../features/tunnel/hooks';

export function useAutomaticLocationName(): string | undefined {
  const {
    status: { state },
    entryHostname,
  } = useConnection();

  const { relayLocations } = useRelayLocations();
  const disconnectedLabel = messages.gettext('Automatic');

  if (!entryHostname) {
    return disconnectedLabel;
  }

  const relay = findRelay(entryHostname, relayLocations);
  if (!relay) {
    return disconnectedLabel;
  }

  const country = findCountryForRelay(entryHostname, relayLocations);

  if (!country) {
    return disconnectedLabel;
  }

  const connectedLabel = sprintf(messages.gettext('Automatic (%(location)s)'), {
    location: country?.name,
  });

  return state === 'connected' || state === 'connecting' ? connectedLabel : disconnectedLabel;
}
