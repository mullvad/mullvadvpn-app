import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import AccountExpiry from '../../shared/account-expiry';
import NewAccountView from '../components/NewAccountView';
import userInterfaceActions from '../redux/userinterface/actions';

import withAppContext, { IAppContext } from '../context';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState) => ({
  accountToken: state.account.accountToken,
  accountExpiry: state.account.expiry
    ? new AccountExpiry(state.account.expiry, state.userInterface.locale)
    : undefined,
  isOffline: state.connection.isBlocked,
});
const mapDispatchToProps = (dispatch: ReduxDispatch, props: IAppContext) => {
  const userInterface = bindActionCreators(userInterfaceActions, dispatch);
  return {
    updateAccountData: () => props.app.updateAccountData(),
    hideWelcomeView: () => userInterface.setShowWelcomeView(false),
    onExternalLinkWithAuth: (url: string) => props.app.openLinkWithAuth(url),
  };
};

export default withAppContext(connect(mapStateToProps, mapDispatchToProps)(NewAccountView));
