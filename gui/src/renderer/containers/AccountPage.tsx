import { connect } from 'react-redux';
import { RouteComponentProps, withRouter } from 'react-router';
import { links } from '../../config.json';
import consumePromise from '../../shared/promise';
import Account from '../components/Account';

import withAppContext, { IAppContext } from '../context';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState) => ({
  accountToken: state.account.accountToken,
  accountExpiry: state.account.expiry,
  expiryLocale: state.userInterface.locale,
  isOffline: state.connection.isBlocked,
});
const mapDispatchToProps = (_dispatch: ReduxDispatch, props: RouteComponentProps & IAppContext) => {
  return {
    onLogout: () => {
      consumePromise(props.app.logout());
    },
    onClose: () => {
      props.history.goBack();
    },
    onBuyMore: () => props.app.openLinkWithAuth(links.purchase),
  };
};

export default withAppContext(withRouter(connect(mapStateToProps, mapDispatchToProps)(Account)));
