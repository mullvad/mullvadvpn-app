import { useEffect, useState } from 'react';
import { hasExpired } from '../../shared/account-expiry';
import { useSelector } from '../redux/store';
import ConnectPage from '../containers/ConnectPage';
import ExpiredAccountErrorViewContainer from '../containers/ExpiredAccountErrorViewContainer';
import { useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';

export default function MainView() {
  const history = useHistory();
  const accountExpiry = useSelector((state) => state.account.expiry);
  const accountHasExpired = accountExpiry !== undefined && hasExpired(accountExpiry);
  const isNewAccount = useSelector(
    (state) => state.account.status.type === 'ok' && state.account.status.method === 'new_account',
  );

  const [showAccountExpired, setShowAccountExpired] = useState<boolean>(
    isNewAccount || accountHasExpired,
  );

  useEffect(() => {
    if (accountHasExpired) {
      setShowAccountExpired(true);
    } else if (showAccountExpired && !accountHasExpired) {
      history.push(RoutePath.timeAdded);
    }
  }, [showAccountExpired, accountHasExpired]);

  if (showAccountExpired) {
    return <ExpiredAccountErrorViewContainer />;
  } else {
    return <ConnectPage />;
  }
}
