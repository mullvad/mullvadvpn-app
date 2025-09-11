import { useSelector } from '../../../../../../../redux/store';
import { ConnectButton, DisconnectButton } from '../';

export function ConnectionActionButton() {
  const tunnelState = useSelector((state) => state.connection.status.state);

  if (tunnelState === 'disconnected' || tunnelState === 'disconnecting') {
    return <ConnectButton disabled={tunnelState === 'disconnecting'} />;
  } else {
    return <DisconnectButton />;
  }
}
