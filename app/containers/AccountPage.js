// @flow

import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'react-router-redux';
import Account from '../components/Account';
import { links } from '../config';
import { openLink } from '../lib/platform';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => ({
  accountToken: state.account.accountToken,
  accountExpiry: state.account.expiry,
});
const mapDispatchToProps = (dispatch: ReduxDispatch, props: SharedRouteProps) => {
  const { push: pushHistory } = bindActionCreators({ push }, dispatch);
  return {
    updateAccountExpiry: () => props.app.updateAccountExpiry(),
    onLogout: () => {
      props.app.logout();
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
