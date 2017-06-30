import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'react-router-redux';
import Settings from '../components/Settings';
import settingsActions from '../actions/settings';
import { remote, shell } from 'electron';
import { links } from '../config';

const mapStateToProps = (state) => {
  return state;
};

const mapDispatchToProps = (dispatch, _props) => {
  const { updateSettings } = bindActionCreators(settingsActions, dispatch);
  return {
    onQuit: () => remote.app.quit(),
    onClose: () => dispatch(push('/connect')),
    onViewAccount: () => dispatch(push('/settings/account')),
    onExternalLink: (type) => shell.openExternal(links[type]),
    onUpdateSettings: updateSettings
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Settings);
