import { connect } from 'react-redux';
import { push } from 'react-router-redux';
import Settings from '../components/Settings';
import { remote, shell } from 'electron';
import { links } from '../config';

const mapStateToProps = (state) => {
  return state;
};

const mapDispatchToProps = (dispatch, _props) => {
  return {
    onQuit: () => remote.app.quit(),
    onClose: () => dispatch(push('/connect')),
    onViewAccount: () => dispatch(push('/settings/account')),
    onExternalLink: (type) => shell.openExternal(links[type]),
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Settings);
