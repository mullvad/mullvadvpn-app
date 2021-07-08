import { connect } from 'react-redux';
import Settings from '../components/Settings';
import withAppContext, { IAppContext } from '../context';
import { IHistoryProps, withHistory } from '../lib/history';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState, props: IAppContext) => ({
  preferredLocaleDisplayName: props.app.getPreferredLocaleDisplayName(
    state.settings.guiSettings.preferredLocale,
  ),
  loginState: state.account.status,
  accountExpiry: state.account.expiry,
  appVersion: state.version.current,
  consistentVersion: state.version.consistent,
  upToDateVersion: state.version.suggestedUpgrade ? false : true,
  suggestedIsBeta: state.version.suggestedIsBeta ?? false,
  isOffline: state.connection.isBlocked,
});
const mapDispatchToProps = (_dispatch: ReduxDispatch, props: IHistoryProps & IAppContext) => {
  return {
    onQuit: () => props.app.quit(),
    onClose: () => props.history.dismiss(),
    onViewSelectLanguage: () => props.history.push('/settings/language'),
    onViewAccount: () => props.history.push('/settings/account'),
    onViewSupport: () => props.history.push('/settings/support'),
    onViewPreferences: () => props.history.push('/settings/preferences'),
    onViewAdvancedSettings: () => props.history.push('/settings/advanced'),
    onExternalLink: (url: string) => props.app.openUrl(url),
    updateAccountData: () => props.app.updateAccountData(),
  };
};

export default withAppContext(withHistory(connect(mapStateToProps, mapDispatchToProps)(Settings)));
