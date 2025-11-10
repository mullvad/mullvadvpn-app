import React from 'react';

import log from '../../../../../shared/logging';
import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useRelaySettingsUpdater } from '../../../../lib/constraint-updater';
import { useNormalRelaySettings } from '../../../../lib/relay-settings-hooks';
import { useMultihop } from '../../hooks';

export type MultihopSwitchProps = SwitchProps;

function MultihopSwitch({ children, ...props }: MultihopSwitchProps) {
  const multihop = useMultihop();
  const normalRelaySettings = useNormalRelaySettings();
  const relaySettingsUpdater = useRelaySettingsUpdater();

  const unavailable = normalRelaySettings === null;

  const setMultihop = React.useCallback(
    async (enabled: boolean) => {
      try {
        await relaySettingsUpdater((settings) => {
          settings.wireguardConstraints.useMultihop = enabled;
          return settings;
        });
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update WireGuard multihop settings', error.message);
      }
    },
    [relaySettingsUpdater],
  );

  return (
    <Switch
      disabled={unavailable}
      checked={multihop && !unavailable}
      onCheckedChange={setMultihop}
      {...props}>
      {children}
    </Switch>
  );
}

const MultihopSwitchNamespace = Object.assign(MultihopSwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { MultihopSwitchNamespace as MultihopSwitch };
