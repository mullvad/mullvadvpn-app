import { goBack } from 'connected-react-router';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { links } from '../../config.json';
import Account from '../components/Account';

import withAppContext, { IAppContext } from '../context';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState) => ({
  accountToken: state.account.accountToken,
  accountExpiry: state.account.expiry,
  expiryLocale: state.userInterface.locale,
  isOffline: state.connection.isBlocked,
});
const mapDispatchToProps = (dispatch: ReduxDispatch, props: IAppContext) => {
  const history = bindActionCreators({ goBack }, dispatch);
  return {
    onLogout: () => {
      props.app.logout();
    },
    onClose: () => {
      history.goBack();
    },
    onBuyMore: () => props.app.openLinkWithAuth(links.purchase),
  };
};

export default withAppContext(connect(mapStateToProps, mapDispatchToProps)(Account));
