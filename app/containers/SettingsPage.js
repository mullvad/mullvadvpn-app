// @flow

import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'connected-react-router';
import Settings from '../components/Settings';
import { links } from '../config';
import { getAppVersion, openLink, exit } from '../lib/platform';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => ({
  account: state.account,
  settings: state.settings,
  version: getAppVersion(),
});
const mapDispatchToProps = (dispatch: ReduxDispatch, _props: SharedRouteProps) => {
  const { push: pushHistory } = bindActionCreators({ push }, dispatch);
  return {
    onQuit: () => exit(),
    onClose: () => pushHistory('/connect'),
    onViewAccount: () => pushHistory('/settings/account'),
    onViewSupport: () => pushHistory('/settings/support'),
    onViewPreferences: () => pushHistory('/settings/preferences'),
    onViewAdvancedSettings: () => pushHistory('/settings/advanced'),
    onExternalLink: (type) => openLink(links[type]),
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(Settings);
