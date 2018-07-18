// @flow
import { shell } from 'electron';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'connected-react-router';
import Login from '../components/Login';
import accountActions from '../redux/account/actions';
import { links } from '../../config';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => {
  const { accountToken, accountHistory, error, status } = state.account;
  return {
    accountToken,
    accountHistory,
    loginError: error,
    loginState: status,
  };
};
const mapDispatchToProps = (dispatch: ReduxDispatch, props: SharedRouteProps) => {
  const history = bindActionCreators({ push }, dispatch);
  const { resetLoginError, updateAccountToken } = bindActionCreators(accountActions, dispatch);
  return {
    openSettings: () => {
      history.push('/settings');
    },
    login: (account) => {
      props.app.login(account);
    },
    resetLoginError: () => {
      resetLoginError();
    },
    openExternalLink: (type) => shell.openExternal(links[type]),
    updateAccountToken: updateAccountToken,
    removeAccountTokenFromHistory: (token) => props.app.removeAccountFromHistory(token),
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(Login);
