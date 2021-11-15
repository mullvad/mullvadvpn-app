import { useEffect, useState } from 'react';
import { hasExpired } from '../../shared/account-expiry';
import { useSelector } from '../redux/store';
import ConnectPage from '../containers/ConnectPage';
import ExpiredAccountErrorViewContainer from '../containers/ExpiredAccountErrorViewContainer';
import { useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { DeviceRevokedView } from './DeviceRevokedView';

export default function MainView() {
  const history = useHistory();
  const accountExpiry = useSelector((state) => state.account.expiry);
  const accountHasExpired = accountExpiry !== undefined && hasExpired(accountExpiry);
  const isNewAccount = useSelector(
    (state) => state.account.status.type === 'ok' && state.account.status.method === 'new_account',
  );
  const showDeviceRevoked = useSelector(
    (state) =>
      (state.connection.status.state === 'error' &&
        state.connection.status.details.cause.reason === 'tunnel_parameter_error' &&
        state.connection.status.details.cause.details === 'no_wireguard_key') ||
      (state.account.status.type === 'none' && state.account.status.deviceRevoked),
  );

  const [showAccountExpired, setShowAccountExpired] = useState<boolean>(
    (isNewAccount || accountHasExpired) && !showDeviceRevoked,
  );

  useEffect(() => {
    if (accountHasExpired && !showDeviceRevoked) {
      setShowAccountExpired(true);
    } else if (showAccountExpired && !accountHasExpired) {
      history.push(RoutePath.timeAdded);
    }
  }, [showAccountExpired, accountHasExpired]);

  if (showDeviceRevoked) {
    return <DeviceRevokedView />;
  } else if (showAccountExpired) {
    return <ExpiredAccountErrorViewContainer />;
  } else {
    return <ConnectPage />;
  }
}
