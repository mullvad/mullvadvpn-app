import { push } from 'connected-react-router';
import { shell } from 'electron';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import Login from '../components/Login';
import withAppContext, { IAppContext } from '../context';
import accountActions from '../redux/account/actions';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState) => {
  const { accountToken, accountHistory, error, status } = state.account;
  return {
    accountToken,
    accountHistory,
    loginError: error,
    loginState: status,
  };
};
const mapDispatchToProps = (dispatch: ReduxDispatch, props: IAppContext) => {
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
    updateAccountToken,
    removeAccountTokenFromHistory: (token: string) => props.app.removeAccountFromHistory(token),
  };
};

export default withAppContext(connect(mapStateToProps, mapDispatchToProps)(Login));
