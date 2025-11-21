import React from 'react';

import log from '../../../../../shared/logging';
import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useNormalRelaySettings } from '../../../../lib/relay-settings-hooks';
import { useMultihop } from '../../hooks';

export type MultihopSwitchProps = SwitchProps;

function MultihopSwitch({ children, ...props }: MultihopSwitchProps) {
  const { multihop, setMultihop: setMultihopImpl } = useMultihop();

  const normalRelaySettings = useNormalRelaySettings();
  const unavailable = normalRelaySettings === null;
  const checked = multihop && !unavailable;

  const setMultihop = React.useCallback(
    async (enabled: boolean) => {
      try {
        await setMultihopImpl(enabled);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update WireGuard multihop settings', error.message);
      }
    },
    [setMultihopImpl],
  );

  return (
    <Switch disabled={unavailable} checked={checked} onCheckedChange={setMultihop} {...props}>
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
