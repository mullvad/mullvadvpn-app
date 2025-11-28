import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useUnpinnedWindow } from '../../hooks';

export type UnpinnedWindowSwitchProps = SwitchProps;

function UnpinnedWindowSwitch({ children, ...props }: UnpinnedWindowSwitchProps) {
  const { unpinnedWindow, setUnpinnedWindow } = useUnpinnedWindow();

  return (
    <Switch checked={unpinnedWindow} onCheckedChange={setUnpinnedWindow} {...props}>
      {children}
    </Switch>
  );
}

const UnpinnedWindowSwitchNamespace = Object.assign(UnpinnedWindowSwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { UnpinnedWindowSwitchNamespace as UnpinnedWindowSwitch };
