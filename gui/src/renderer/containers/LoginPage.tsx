import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import Login from '../components/Login';
import withAppContext, { IAppContext } from '../context';
import accountActions from '../redux/account/actions';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState) => {
  const { accountToken, accountHistory, status } = state.account;
  return {
    accountToken,
    accountHistory,
    loginState: status,
  };
};
const mapDispatchToProps = (dispatch: ReduxDispatch, props: IAppContext) => {
  const { resetLoginError, updateAccountToken } = bindActionCreators(accountActions, dispatch);
  return {
    login: (account: string) => {
      void props.app.login(account);
    },
    resetLoginError: () => {
      resetLoginError();
    },
    openExternalLink: (url: string) => props.app.openUrl(url),
    updateAccountToken,
    clearAccountHistory: () => props.app.clearAccountHistory(),
    createNewAccount: () => void props.app.createNewAccount(),
  };
};

export default withAppContext(connect(mapStateToProps, mapDispatchToProps)(Login));
