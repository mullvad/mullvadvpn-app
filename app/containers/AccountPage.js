import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'react-router-redux';
import Account from '../components/Account';
import accountActions from '../redux/account/actions';
import { shell } from 'electron';
import { links } from '../config';

const mapStateToProps = (state) => {
  return state;
};

const mapDispatchToProps = (dispatch, props) => {
  const { logout } = bindActionCreators(accountActions, dispatch);
  return {
    onLogout: () => logout(props.backend),
    onClose: () => dispatch(push('/settings')),
    onViewAccount: () => dispatch(push('/settings/account')),
    onExternalLink: (type) => shell.openExternal(links[type])
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Account);
