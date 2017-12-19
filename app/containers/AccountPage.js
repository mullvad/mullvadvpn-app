// @flow

import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'react-router-redux';
import Account from '../components/Account';
import accountActions from '../redux/account/actions';
import { shell } from 'electron';
import { links } from '../config';

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
    onBuyMore: () => shell.openExternal(links['purchase'])
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Account);
