import React, { useEffect, useState } from 'react';
import { useSelector } from 'react-redux';
import { hasExpired } from '../../shared/account-expiry';
import { IReduxState } from '../redux/store';
import ConnectPage from '../containers/ConnectPage';
import ExpiredAccountErrorViewContainer from '../containers/ExpiredAccountErrorViewContainer';
import { useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';

export default function MainView() {
  const history = useHistory();
  const accountExpiry = useSelector((state: IReduxState) => state.account.expiry);
  const accountHasExpired = accountExpiry && hasExpired(accountExpiry);
  const isNewAccount = useSelector(
    (state: IReduxState) =>
      state.account.status.type === 'ok' && state.account.status.method === 'new_account',
  );
  const [showAccountExpired, setShowAccountExpired] = useState(isNewAccount || accountHasExpired);

  useEffect(() => {
    if (accountHasExpired) {
      setShowAccountExpired(true);
    } else if (showAccountExpired && !accountHasExpired) {
      history.push(RoutePath.timeAdded);
    }
  }, [showAccountExpired, accountHasExpired]);

  return showAccountExpired ? <ExpiredAccountErrorViewContainer /> : <ConnectPage />;
}
