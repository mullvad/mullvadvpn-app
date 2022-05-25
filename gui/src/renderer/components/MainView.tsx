import { useEffect, useState } from 'react';

import { hasExpired } from '../../shared/account-expiry';
import ConnectPage from '../containers/ConnectPage';
import ExpiredAccountErrorViewContainer from '../containers/ExpiredAccountErrorViewContainer';
import { useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { useSelector } from '../redux/store';

type ExpiryData = { show: false } | { show: true; expiry: string | undefined };

export default function MainView() {
  const history = useHistory();
  const accountExpiry = useSelector((state) => state.account.expiry);
  const accountHasExpired = accountExpiry !== undefined && hasExpired(accountExpiry);
  const isNewAccount = useSelector(
    (state) => state.account.status.type === 'ok' && state.account.status.method === 'new_account',
  );

  const [showAccountExpired, setShowAccountExpired] = useState<ExpiryData>(() =>
    isNewAccount || accountHasExpired ? { show: true, expiry: accountExpiry } : { show: false },
  );

  useEffect(() => {
    if (
      accountHasExpired &&
      (!showAccountExpired.show || showAccountExpired.expiry !== accountExpiry)
    ) {
      setShowAccountExpired({ show: true, expiry: accountExpiry });
    } else if (
      showAccountExpired.show &&
      accountExpiry !== showAccountExpired.expiry &&
      !accountHasExpired
    ) {
      history.push(RoutePath.timeAdded);
    }
  }, [showAccountExpired, accountHasExpired]);

  if (showAccountExpired.show) {
    return <ExpiredAccountErrorViewContainer />;
  } else {
    return <ConnectPage />;
  }
}
