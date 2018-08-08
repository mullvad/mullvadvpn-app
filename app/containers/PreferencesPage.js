// @flow

import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'connected-react-router';
import Preferences from '../components/Preferences';
import { log, getOpenAtLogin, setOpenAtLogin } from '../lib/platform';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => ({
  autoConnect: state.settings.autoConnect,
  allowLan: state.settings.allowLan,
});

const mapDispatchToProps = (dispatch: ReduxDispatch, props: SharedRouteProps) => {
  const { push: pushHistory } = bindActionCreators({ push }, dispatch);
  return {
    onClose: () => pushHistory('/settings'),
    getAutoStart: () => {
      return getOpenAtLogin();
    },
    setAutoStart: async (autoStart) => {
      try {
        await setOpenAtLogin(autoStart);
      } catch (error) {
        log.error(`Cannot set auto-start: ${error.message}`);
      }
    },
    setAutoConnect: async (autoConnect) => {
      try {
        props.app.setAutoConnect(autoConnect);
      } catch (error) {
        log.error(`Cannot set auto-connect: ${error.message}`);
      }
    },
    setAllowLan: (allowLan) => {
      props.app.setAllowLan(allowLan);
    },
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(Preferences);
