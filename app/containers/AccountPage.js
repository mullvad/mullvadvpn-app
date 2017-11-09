// @flow

import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'react-router-redux';
import Account from '../components/Account';
import accountActions from '../redux/account/actions';
import { links } from '../config';
import Linking from '../lib/linking';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => state;
const mapDispatchToProps = (dispatch: ReduxDispatch, props: SharedRouteProps) => {
  const { push: pushHistory } = bindActionCreators({ push }, dispatch);
  const { logout } = bindActionCreators(accountActions, dispatch);
  return {
    onLogout: () => {
      logout(props.backend);
    },
    onClose: () => {
      pushHistory('/settings');
    },
    onBuyMore: () => Linking(links['purchase'])
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Account);
