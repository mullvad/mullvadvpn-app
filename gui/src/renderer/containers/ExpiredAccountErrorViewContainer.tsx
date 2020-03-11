import log from 'electron-log';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import AccountExpiry from '../../shared/account-expiry';
import ExpiredAccountErrorView from '../components/ExpiredAccountErrorView';
import accountActions from '../redux/account/actions';

import withAppContext, { IAppContext } from '../context';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState) => ({
  accountToken: state.account.accountToken,
  accountExpiry: state.account.expiry
    ? new AccountExpiry(state.account.expiry, state.userInterface.locale)
    : undefined,
  loginState: state.account.status,
  isBlocked: state.connection.isBlocked,
  blockWhenDisconnected: state.settings.blockWhenDisconnected,
});
const mapDispatchToProps = (dispatch: ReduxDispatch, props: IAppContext) => {
  const account = bindActionCreators(accountActions, dispatch);
  return {
    // Changes login method from "new_account" to "existing_account"
    hideWelcomeView: () => account.loggedIn(),
    onExternalLinkWithAuth: (url: string) => props.app.openLinkWithAuth(url),
    onDisconnect: async () => {
      try {
        await props.app.disconnectTunnel();
      } catch (error) {
        log.error(`Failed to disconnect the tunnel: ${error.message}`);
      }
    },
    setBlockWhenDisconnected: async (blockWhenDisconnected: boolean) => {
      try {
        await props.app.setBlockWhenDisconnected(blockWhenDisconnected);
      } catch (e) {
        log.error('Failed to update block when disconnected', e.message);
      }
    },
  };
};

export default withAppContext(
  connect(mapStateToProps, mapDispatchToProps)(ExpiredAccountErrorView),
);
