import { useSplitTunneling } from '../../../../../../../../../features/split-tunneling/hooks';
import { Switch } from '../../../../../../../../../lib/components/switch';
import { useDisabled } from './hooks';

export function SplitTunnelingStateSwitch() {
  const { splitTunnelingEnabled, setSplitTunnelingState } = useSplitTunneling();
  const disabled = useDisabled();

  return (
    <Switch
      checked={splitTunnelingEnabled}
      disabled={disabled}
      onCheckedChange={setSplitTunnelingState}>
      <Switch.Trigger>
        <Switch.Thumb />
      </Switch.Trigger>
    </Switch>
  );
}
