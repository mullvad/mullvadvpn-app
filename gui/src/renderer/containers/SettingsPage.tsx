import { connect } from 'react-redux';
import { RouteComponentProps, withRouter } from 'react-router';
import Settings from '../components/Settings';
import withAppContext, { IAppContext } from '../context';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState, props: IAppContext) => ({
  preferredLocaleDisplayName: props.app.getPreferredLocaleDisplayName(
    state.settings.guiSettings.preferredLocale,
  ),
  loginState: state.account.status,
  accountExpiry: state.account.expiry,
  expiryLocale: state.userInterface.locale,
  appVersion: state.version.current,
  consistentVersion: state.version.consistent,
  upToDateVersion: state.version.suggestedUpgrade ? false : true,
  isOffline: state.connection.isBlocked,
});
const mapDispatchToProps = (_dispatch: ReduxDispatch, props: RouteComponentProps & IAppContext) => {
  return {
    onQuit: () => props.app.quit(),
    onClose: () => props.history.goBack(),
    onViewSelectLanguage: () => props.history.push('/settings/language'),
    onViewAccount: () => props.history.push('/settings/account'),
    onViewSupport: () => props.history.push('/settings/support'),
    onViewPreferences: () => props.history.push('/settings/preferences'),
    onViewAdvancedSettings: () => props.history.push('/settings/advanced'),
    onExternalLink: (url: string) => props.app.openUrl(url),
  };
};

export default withAppContext(withRouter(connect(mapStateToProps, mapDispatchToProps)(Settings)));
