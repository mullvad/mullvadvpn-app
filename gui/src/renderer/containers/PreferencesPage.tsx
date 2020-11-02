import log from 'electron-log';
import { connect } from 'react-redux';
import { RouteComponentProps, withRouter } from 'react-router';
import consumePromise from '../../shared/promise';
import Preferences from '../components/Preferences';
import withAppContext, { IAppContext } from '../context';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState) => ({
  autoStart: state.settings.autoStart,
  allowLan: state.settings.allowLan,
  showBetaReleases: state.settings.showBetaReleases,
  isBeta: state.version.isBeta,
  autoConnect: state.settings.guiSettings.autoConnect,
  enableSystemNotifications: state.settings.guiSettings.enableSystemNotifications,
  monochromaticIcon: state.settings.guiSettings.monochromaticIcon,
  startMinimized: state.settings.guiSettings.startMinimized,
});

const mapDispatchToProps = (_dispatch: ReduxDispatch, props: RouteComponentProps & IAppContext) => {
  return {
    onClose: () => {
      props.history.goBack();
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
      consumePromise(props.app.setAllowLan(allowLan));
    },
    setShowBetaReleases: (showBetaReleases: boolean) => {
      consumePromise(props.app.setShowBetaReleases(showBetaReleases));
    },
    setStartMinimized: (startMinimized: boolean) => {
      props.app.setStartMinimized(startMinimized);
    },
    enableStartMinimizedToggle: process.platform === 'linux',
    setMonochromaticIcon: (monochromaticIcon: boolean) => {
      props.app.setMonochromaticIcon(monochromaticIcon);
    },
  };
};

export default withAppContext(
  withRouter(connect(mapStateToProps, mapDispatchToProps)(Preferences)),
);
