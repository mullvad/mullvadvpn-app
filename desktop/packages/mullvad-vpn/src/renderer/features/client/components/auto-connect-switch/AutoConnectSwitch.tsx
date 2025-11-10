import { useAppContext } from '../../../../context';
import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useAutoConnect } from '../../hooks';

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
