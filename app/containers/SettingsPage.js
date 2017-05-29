import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'react-router-redux';
import Settings from '../components/Settings';
import userActions from '../actions/user';
import settingsActions from '../actions/settings';
import { remote, shell } from 'electron';
import { links } from '../config';

const mapStateToProps = (state) => {
  return state;
};

const mapDispatchToProps = (dispatch, props) => {
  const { logout } = bindActionCreators(userActions, dispatch);
  const { updateSettings } = bindActionCreators(settingsActions, dispatch);
  return {
    onQuit: () => remote.app.quit(),
    onLogout: () => logout(props.backend),
    onClose: () => dispatch(push('/connect')),
    onViewAccount: () => dispatch(push('/settings/account')),
    onExternalLink: (type) => shell.openExternal(links[type]),
    onUpdateSettings: updateSettings
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Settings);
