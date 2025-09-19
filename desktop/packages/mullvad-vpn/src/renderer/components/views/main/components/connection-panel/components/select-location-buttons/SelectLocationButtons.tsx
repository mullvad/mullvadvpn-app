import { useSelector } from '../../../../../../../redux/store';
import { MultiButton, ReconnectButton, SelectLocationButton } from './components';

export function SelectLocationButtons() {
  const tunnelState = useSelector((state) => state.connection.status.state);

  if (tunnelState === 'connecting' || tunnelState === 'connected') {
    return <MultiButton mainButton={SelectLocationButton} sideButton={ReconnectButton} />;
  } else {
    return <SelectLocationButton />;
  }
}
