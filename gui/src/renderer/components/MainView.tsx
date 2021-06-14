import React from 'react';
import { useSelector } from 'react-redux';
import { hasExpired } from '../../shared/account-expiry';
import { IReduxState } from '../redux/store';
import ConnectPage from '../containers/ConnectPage';
import ExpiredAccountErrorViewContainer from '../containers/ExpiredAccountErrorViewContainer';

export default function MainView() {
  const accountExpiry = useSelector((state: IReduxState) => state.account.expiry);
  const accountExpired = accountExpiry && hasExpired(accountExpiry);

  return accountExpired ? <ExpiredAccountErrorViewContainer /> : <ConnectPage />;
}
