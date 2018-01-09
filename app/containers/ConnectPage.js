// @flow

import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'react-router-redux';
import { links } from '../config';
import Connect from '../components/Connect';
import connectActions from '../redux/connection/actions';
import { openLink } from '../lib/platform';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => {
  return {
    accountExpiry: state.account.expiry,
    connection: state.connection,
    settings: state.settings,
  };
};

const mapDispatchToProps = (dispatch: ReduxDispatch, props: SharedRouteProps) => {
  const { connect, disconnect, copyIPAddress } = bindActionCreators(connectActions, dispatch);
  const { push: pushHistory } = bindActionCreators({ push }, dispatch);
  const { backend } = props;

  return {
    onSettings: () => {
      pushHistory('/settings');
    },
    onSelectLocation: () => {
      pushHistory('/select-location');
    },
    onConnect: () => {
      connect(backend);
    },
    onCopyIP: () => {
      copyIPAddress();
    },
    onDisconnect: () => {
      disconnect(backend);
    },
    onExternalLink: (type) => openLink(links[type]),
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Connect);
