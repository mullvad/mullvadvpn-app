import { goBack, push } from 'connected-react-router';
import { remote, shell } from 'electron';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import Settings from '../components/Settings';

import { IReduxState, ReduxDispatch } from '../redux/store';
import { ISharedRouteProps } from '../routes';

const mapStateToProps = (state: IReduxState) => ({
  loginState: state.account.status,
  accountExpiry: state.account.expiry,
  appVersion: state.version.current,
  consistentVersion: state.version.consistent,
  upToDateVersion: state.version.upToDate,
  isOffline: state.connection.isBlocked,
});
const mapDispatchToProps = (dispatch: ReduxDispatch, _props: ISharedRouteProps) => {
  const history = bindActionCreators({ push, goBack }, dispatch);
  return {
    onQuit: () => remote.app.quit(),
    onClose: () => history.goBack(),
    onViewAccount: () => history.push('/settings/account'),
    onViewSupport: () => history.push('/settings/support'),
    onViewPreferences: () => history.push('/settings/preferences'),
    onViewAdvancedSettings: () => history.push('/settings/advanced'),
    onExternalLink: (url: string) => shell.openExternal(url),
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(Settings);
