import { connect } from 'react-redux';
import { IDnsOptions } from '../../shared/daemon-rpc-types';
import log from '../../shared/logging';
import Preferences from '../components/Preferences';
import withAppContext, { IAppContext } from '../context';
import { IHistoryProps, withHistory } from '../lib/history';
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
  unpinnedWindow: state.settings.guiSettings.unpinnedWindow,
  dns: state.settings.dns,
});

const mapDispatchToProps = (_dispatch: ReduxDispatch, props: IHistoryProps & IAppContext) => {
  return {
    onClose: () => {
      props.history.pop();
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
      void props.app.setAllowLan(allowLan);
    },
    setShowBetaReleases: (showBetaReleases: boolean) => {
      void props.app.setShowBetaReleases(showBetaReleases);
    },
    setStartMinimized: (startMinimized: boolean) => {
      props.app.setStartMinimized(startMinimized);
    },
    setMonochromaticIcon: (monochromaticIcon: boolean) => {
      props.app.setMonochromaticIcon(monochromaticIcon);
    },
    setUnpinnedWindow: (unpinnedWindow: boolean) => {
      props.app.setUnpinnedWindow(unpinnedWindow);
    },
    setDnsOptions: (dns: IDnsOptions) => {
      return props.app.setDnsOptions(dns);
    },
  };
};

export default withAppContext(
  withHistory(connect(mapStateToProps, mapDispatchToProps)(Preferences)),
);
