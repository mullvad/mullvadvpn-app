import { useAppContext } from '../../../../../../../../../context';
import { Switch } from '../../../../../../../../../lib/components/switch';
import { useSelector } from '../../../../../../../../../redux/store';
import { useDisabled } from './hooks';

export function SplitTunnelingStateSwitch() {
  const { setSplitTunnelingState } = useAppContext();
  const disabled = useDisabled();
  const splitTunnelingEnabled = useSelector((state) => state.settings.splitTunneling);

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
