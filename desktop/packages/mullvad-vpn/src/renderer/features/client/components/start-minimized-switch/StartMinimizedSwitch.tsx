import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useStartMinimized } from '../../hooks';

export type StartMinimizedSwitchProps = SwitchProps;

function StartMinimizedSwitch({ children, ...props }: StartMinimizedSwitchProps) {
  const { startMinimized, setStartMinimized } = useStartMinimized();

  return (
    <Switch checked={startMinimized} onCheckedChange={setStartMinimized} {...props}>
      {children}
    </Switch>
  );
}

const StartMinimizedSwitchNamespace = Object.assign(StartMinimizedSwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { StartMinimizedSwitchNamespace as StartMinimizedSwitch };
