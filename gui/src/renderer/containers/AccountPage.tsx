import { goBack } from 'connected-react-router';
import { shell } from 'electron';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { links } from '../../config.json';
import Account from '../components/Account';

import { IReduxState, ReduxDispatch } from '../redux/store';
import { ISharedRouteProps } from '../routes';

const mapStateToProps = (state: IReduxState, props: ISharedRouteProps) => ({
  accountToken: state.account.accountToken,
  accountExpiry: state.account.expiry,
  expiryLocale: props.locale,
  isOffline: state.connection.isBlocked,
});
const mapDispatchToProps = (dispatch: ReduxDispatch, props: ISharedRouteProps) => {
  const history = bindActionCreators({ goBack }, dispatch);
  return {
    onLogout: () => {
      props.app.logout();
    },
    onClose: () => {
      history.goBack();
    },
    onBuyMore: () => shell.openExternal(links.purchase),
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(Account);
