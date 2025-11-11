import React from 'react';

import log from '../../../../../shared/logging';
import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useAutoStart } from '../../hooks';

export type AutoStartSwitch = SwitchProps;

function AutoStartSwitch({ children, ...props }: AutoStartSwitch) {
  const { autoStart, setAutoStart: setAutoStartImpl } = useAutoStart();

  const setAutoStart = React.useCallback(
    async (autoStart: boolean) => {
      try {
        await setAutoStartImpl(autoStart);
      } catch (e) {
        const error = e as Error;
        log.error(`Cannot set auto-start: ${error.message}`);
      }
    },
    [setAutoStartImpl],
  );

  return (
    <Switch checked={autoStart} onCheckedChange={setAutoStart} {...props}>
      {children}
    </Switch>
  );
}

const AutoStartSwitchNamespace = Object.assign(AutoStartSwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { AutoStartSwitchNamespace as AutoStartSwitch };
