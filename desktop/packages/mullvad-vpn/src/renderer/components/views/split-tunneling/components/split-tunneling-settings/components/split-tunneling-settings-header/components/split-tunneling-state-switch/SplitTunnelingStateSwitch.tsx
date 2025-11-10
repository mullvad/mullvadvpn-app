import { useAppContext } from '../../../../../../../../../context';
import { useSplitTunneling } from '../../../../../../../../../features/split-tunneling/hooks';
import { Switch } from '../../../../../../../../../lib/components/switch';
import { useDisabled } from './hooks';

export function SplitTunnelingStateSwitch() {
  const { setSplitTunnelingState } = useAppContext();
  const disabled = useDisabled();
  const splitTunnelingEnabled = useSplitTunneling();

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
