import { goBack } from 'connected-react-router';
import log from 'electron-log';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import Preferences from '../components/Preferences';

import { IReduxState, ReduxDispatch } from '../redux/store';
import { ISharedRouteProps } from '../routes';

const mapStateToProps = (state: IReduxState) => ({
  autoStart: state.settings.autoStart,
  allowLan: state.settings.allowLan,
  autoConnect: state.settings.guiSettings.autoConnect,
  enableSystemNotifications: state.settings.guiSettings.enableSystemNotifications,
  monochromaticIcon: state.settings.guiSettings.monochromaticIcon,
  startMinimized: state.settings.guiSettings.startMinimized,
});

const mapDispatchToProps = (dispatch: ReduxDispatch, props: ISharedRouteProps) => {
  const history = bindActionCreators({ goBack }, dispatch);
  return {
    onClose: () => {
      history.goBack();
    },
    setEnableSystemNotifications: (flag: boolean) => {
      props.app.setEnableSystemNotifications(flag);
    },
    setAutoStart: async (autoStart: boolean) => {
      try {
        await props.app.setAutoStart(autoStart);
      } catch (error) {
        log.error(`Cannot set auto-start: ${error.message}`);
      }
    },
    setAutoConnect: (autoConnect: boolean) => {
      props.app.setAutoConnect(autoConnect);
    },
    setAllowLan: (allowLan: boolean) => {
      props.app.setAllowLan(allowLan);
    },
    setStartMinimized: (startMinimized: boolean) => {
      props.app.setStartMinimized(startMinimized);
    },
    enableStartMinimizedToggle: process.platform === 'linux',
    setMonochromaticIcon: (monochromaticIcon: boolean) => {
      props.app.setMonochromaticIcon(monochromaticIcon);
    },
    enableMonochromaticIconToggle: process.platform === 'darwin',
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(Preferences);
