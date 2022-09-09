import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { links } from '../../config.json';
import Account from '../components/Account';
import withAppContext, { IAppContext } from '../context';
import { IHistoryProps, withHistory } from '../lib/history';
import accountActions from '../redux/account/actions';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState) => ({
  isOffline: state.connection.isBlocked,
});
const mapDispatchToProps = (dispatch: ReduxDispatch, props: IHistoryProps & IAppContext) => {
  const account = bindActionCreators(accountActions, dispatch);

  return {
    prepareLogout: () => account.prepareLogout(),
    cancelLogout: () => account.cancelLogout(),
    onLogout: () => {
      void props.app.logout();
    },
    onClose: () => {
      props.history.pop();
    },
    onBuyMore: () => props.app.openLinkWithAuth(links.purchase),
    updateAccountData: () => props.app.updateAccountData(),
    getDeviceState: () => props.app.getDeviceState(),
  };
};

export default withAppContext(withHistory(connect(mapStateToProps, mapDispatchToProps)(Account)));
