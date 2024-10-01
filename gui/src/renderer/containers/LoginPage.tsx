import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import Login from '../components/Login';
import withAppContext, { IAppContext } from '../context';
import accountActions from '../redux/account/actions';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState) => {
  const tunnelState = state.connection.status;
  const blockWhenDisconnected = state.settings.blockWhenDisconnected;
  const { accountNumber, accountHistory, status } = state.account;

  const showBlockMessage = tunnelState.state === 'error' || blockWhenDisconnected;

  const isPerformingPostUpgrade = state.userInterface.isPerformingPostUpgrade;

  return {
    accountNumber,
    accountHistory,
    loginState: status,
    showBlockMessage,
    isPerformingPostUpgrade,
  };
};
const mapDispatchToProps = (dispatch: ReduxDispatch, props: IAppContext) => {
  const { resetLoginError, updateAccountNumber } = bindActionCreators(accountActions, dispatch);
  return {
    login: (account: string) => {
      void props.app.login(account);
    },
    resetLoginError: () => {
      resetLoginError();
    },
    openExternalLink: (url: string) => props.app.openUrl(url),
    updateAccountNumber,
    clearAccountHistory: () => props.app.clearAccountHistory(),
    createNewAccount: () => void props.app.createNewAccount(),
  };
};

export default withAppContext(connect(mapStateToProps, mapDispatchToProps)(Login));
