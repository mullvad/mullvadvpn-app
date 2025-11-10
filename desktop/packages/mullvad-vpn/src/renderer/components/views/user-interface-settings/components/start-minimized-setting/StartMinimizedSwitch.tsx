import { useAppContext } from '../../../../../context';
import { useStartMinimized } from '../../../../../features/client/hooks';
import { Switch, SwitchProps } from '../../../../../lib/components/switch';

export type StartMinimizedSwitchProps = SwitchProps;

function StartMinimizedSwitch({ children, ...props }: StartMinimizedSwitchProps) {
  const startMinimized = useStartMinimized();
  const { setStartMinimized } = useAppContext();

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
