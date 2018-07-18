// @flow

import log from 'electron-log';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { goBack } from 'connected-react-router';
import Preferences from '../components/Preferences';
import { getOpenAtLogin, setOpenAtLogin } from '../lib/autostart';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => ({
  autoConnect: state.settings.autoConnect,
  allowLan: state.settings.allowLan,
});

const mapDispatchToProps = (dispatch: ReduxDispatch, props: SharedRouteProps) => {
  const history = bindActionCreators({ goBack }, dispatch);
  return {
    onClose: () => {
      history.goBack();
    },
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
