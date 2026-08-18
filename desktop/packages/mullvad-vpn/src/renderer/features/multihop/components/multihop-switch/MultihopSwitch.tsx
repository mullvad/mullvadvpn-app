import React from 'react';

import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useNormalRelaySettings } from '../../../../lib/relay-settings-hooks';
import { useMultihop } from '../../hooks';

export type MultihopSwitchProps = SwitchProps;

function MultihopSwitch({ children, ...props }: MultihopSwitchProps) {
  const { multihop, setMultihop } = useMultihop();

  const normalRelaySettings = useNormalRelaySettings();
  const unavailable = normalRelaySettings === null;
  const checked = multihop && !unavailable;

  const handleCheckedChange = React.useCallback(
    async (checked: boolean) => {
      await setMultihop({ enabled: checked });
    },
    [setMultihop],
  );

  return (
    <Switch
      disabled={unavailable}
      checked={checked}
      onCheckedChange={handleCheckedChange}
      {...props}>
      {children}
    </Switch>
  );
}

const MultihopSwitchNamespace = Object.assign(MultihopSwitch, {
  Label: Switch.Label,
  Input: Switch.Input,
  Trigger: Switch.Trigger,
});

export { MultihopSwitchNamespace as MultihopSwitch };
