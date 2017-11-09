// @flow

import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'react-router-redux';
import Settings from '../components/Settings';
import settingsActions from '../redux/settings/actions';
import { links } from '../config';
import Linking from '../lib/linking';
import Exit from '../lib/exit';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => state;
const mapDispatchToProps = (dispatch: ReduxDispatch, _props: SharedRouteProps) => {
  const { push: pushHistory } = bindActionCreators({ push }, dispatch);
  return {
    onQuit: () => Exit(),
    onClose: () => pushHistory('/connect'),
    onViewAccount: () => pushHistory('/settings/account'),
    onViewSupport: () => pushHistory('/settings/support'),
    onViewAdvancedSettings: () => pushHistory('/settings/advanced'),
    onExternalLink: (type) => Linking(links[type]),
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Settings);
