import { shell } from 'electron';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'connected-react-router';
import Login from '../components/Login';
import accountActions from '../redux/account/actions';

import { ReduxState, ReduxDispatch } from '../redux/store';
import { SharedRouteProps } from '../routes';

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
    login: (account: string) => {
      props.app.login(account);
    },
    resetLoginError: () => {
      resetLoginError();
    },
    openExternalLink: (url: string) => shell.openExternal(url),
    updateAccountToken: updateAccountToken,
    removeAccountTokenFromHistory: (token: string) => props.app.removeAccountFromHistory(token),
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(Login);
