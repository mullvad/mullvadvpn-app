import { useAppContext } from '../../../../../../../../../context';
import { useSelector } from '../../../../../../../../../redux/store';
import { Switch } from '../../../../../../../../cell';
import { useDisabled } from './hooks';

export function SplitTunnelingStateSwitch() {
  const { setSplitTunnelingState } = useAppContext();
  const disabled = useDisabled();
  const splitTunnelingEnabled = useSelector((state) => state.settings.splitTunneling);

  return (
    <Switch isOn={splitTunnelingEnabled} disabled={disabled} onChange={setSplitTunnelingState} />
  );
}
