import { useCallback } from 'react';

import { type RelayLocation, wrapConstraint } from '../../../../../shared/daemon-rpc-types';
import { useAppContext } from '../../../../context';
import { useRelaySettingsModifier } from '../../../../lib/constraint-updater';
import { useHistory } from '../../../../lib/history';
import { useOnSelectLocation } from './use-on-select-location';

export function useOnSelectExitLocation() {
  const onSelectLocation = useOnSelectLocation();
  const history = useHistory();
  const relaySettingsModifier = useRelaySettingsModifier();
  const { connectTunnel } = useAppContext();

  const onSelectRelay = useCallback(
    async (relayLocation: RelayLocation) => {
      const settings = relaySettingsModifier((settings) => ({
        ...settings,
        location: wrapConstraint(relayLocation),
      }));
      history.pop();
      await onSelectLocation({ normal: settings });
      await connectTunnel();
    },
    [connectTunnel, history, onSelectLocation, relaySettingsModifier],
  );

  const onSelectSpecial = useCallback((_location: undefined) => {
    throw new Error('relayLocation should never be undefined');
  }, []);

  return [onSelectRelay, onSelectSpecial] as const;
}
