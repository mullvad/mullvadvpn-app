// @flow

import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'react-router-redux';
import Account from '../components/Account';
import accountActions from '../redux/account/actions';
import { links } from '../config';
import { openLink } from '../lib/platform';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => ({
  accountToken: state.account.accountToken,
  accountExpiry: state.account.expiry,
});
const mapDispatchToProps = (dispatch: ReduxDispatch, props: SharedRouteProps) => {
  const { backend } = props;
  const { push: pushHistory } = bindActionCreators({ push }, dispatch);
  const { logout } = bindActionCreators(accountActions, dispatch);

  return {
    updateAccountExpiry: () => backend.updateAccountExpiry(),
    onLogout: () => {
      logout(backend);
    },
    onClose: () => {
      pushHistory('/settings');
    },
    onBuyMore: () => openLink(links['purchase']),
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(Account);
