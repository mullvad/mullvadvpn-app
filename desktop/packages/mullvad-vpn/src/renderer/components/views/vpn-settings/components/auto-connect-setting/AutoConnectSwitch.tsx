import { useAppContext } from '../../../../../context';
import { useAutoConnect } from '../../../../../features/client/hooks';
import { Switch, SwitchProps } from '../../../../../lib/components/switch';

export type AutoConnectSwitch = SwitchProps;

function AutoConnectSwitch({ children, ...props }: AutoConnectSwitch) {
  const autoConnect = useAutoConnect();
  const { setAutoConnect } = useAppContext();

  return (
    <Switch checked={autoConnect} onCheckedChange={setAutoConnect} {...props}>
      {children}
    </Switch>
  );
}

const AutoConnectSwitchNamespace = Object.assign(AutoConnectSwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { AutoConnectSwitchNamespace as AutoConnectSwitch };
