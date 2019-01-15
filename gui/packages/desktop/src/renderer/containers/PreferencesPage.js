// @flow

import log from 'electron-log';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { goBack } from 'connected-react-router';
import Preferences from '../components/Preferences';
import { getOpenAtLogin } from '../lib/autostart';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => ({
  autoConnect: state.settings.guiSettings.autoConnect,
  allowLan: state.settings.allowLan,
  monochromaticIcon: state.settings.guiSettings.monochromaticIcon,
  startMinimized: state.settings.guiSettings.startMinimized,
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
        await props.app.setAutoStart(autoStart);
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
    setStartMinimized: (startMinimized) => {
      props.app.setStartMinimized(startMinimized);
    },
    enableStartMinimizedToggle: process.platform === 'linux',
    setMonochromaticIcon: (monochromaticIcon) => {
      props.app.setMonochromaticIcon(monochromaticIcon);
    },
    enableMonochromaticIconToggle: process.platform === 'darwin',
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(Preferences);
