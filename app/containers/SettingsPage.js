// @flow

import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push, goBack } from 'connected-react-router';
import Settings from '../components/Settings';
import { links } from '../config';
import { getAppVersion, openLink, exit } from '../lib/platform';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => ({
  loginState: state.account.status,
  accountExpiry: state.account.expiry,
  appVersion: getAppVersion(),
});
const mapDispatchToProps = (dispatch: ReduxDispatch, props: SharedRouteProps) => {
  const history = bindActionCreators({ push, goBack }, dispatch);
  return {
    onQuit: () => exit(),
    onClose: () => history.goBack(),
    onViewAccount: () => history.push('/settings/account'),
    onViewSupport: () => history.push('/settings/support'),
    onViewPreferences: () => history.push('/settings/preferences'),
    onViewAdvancedSettings: () => history.push('/settings/advanced'),
    onExternalLink: (type) => openLink(links[type]),
    updateAccountExpiry: () => props.app.updateAccountExpiry(),
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(Settings);
