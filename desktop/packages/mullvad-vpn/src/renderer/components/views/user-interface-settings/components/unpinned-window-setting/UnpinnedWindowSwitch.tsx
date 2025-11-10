import { useAppContext } from '../../../../../context';
import { useUnpinnedWindow } from '../../../../../features/client/hooks';
import { Switch, SwitchProps } from '../../../../../lib/components/switch';

export type UnpinnedWindowSwitchProps = SwitchProps;

function UnpinnedWindowSwitch({ children, ...props }: UnpinnedWindowSwitchProps) {
  const unpinnedWindow = useUnpinnedWindow();
  const { setUnpinnedWindow } = useAppContext();
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
