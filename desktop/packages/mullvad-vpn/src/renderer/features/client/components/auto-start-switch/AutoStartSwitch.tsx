import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useAutoStart } from '../../hooks';

export type AutoStartSwitch = SwitchProps;

function AutoStartSwitch({ children, ...props }: AutoStartSwitch) {
  const { autoStart, setAutoStart } = useAutoStart();

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
