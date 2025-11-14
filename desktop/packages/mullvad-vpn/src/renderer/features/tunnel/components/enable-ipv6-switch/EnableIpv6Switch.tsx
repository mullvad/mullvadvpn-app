import React from 'react';

import log from '../../../../../shared/logging';
import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useEnableIpv6 } from '../../hooks';

export type EnableIpv6SwitchProps = SwitchProps;

function EnableIpv6Switch({ children, ...props }: EnableIpv6SwitchProps) {
  const { enableIpv6, setEnableIpv6: setEnableIpv6Impl } = useEnableIpv6();

  const setEnableIpv6 = React.useCallback(
    async (enableIpv6: boolean) => {
      try {
        await setEnableIpv6Impl(enableIpv6);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update enable IPv6', error.message);
      }
    },
    [setEnableIpv6Impl],
  );

  return (
    <>
      <Switch checked={enableIpv6} onCheckedChange={setEnableIpv6} {...props}>
        {children}
      </Switch>
    </>
  );
}

const EnableIpv6SwitchNamespace = Object.assign(EnableIpv6Switch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { EnableIpv6SwitchNamespace as EnableIpv6Switch };
