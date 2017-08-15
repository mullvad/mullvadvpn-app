import { shell } from 'electron';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'react-router-redux';
import Login from '../components/Login';
import accountActions from '../redux/account/actions';
import { links } from '../config';

const mapStateToProps = (state) => state;
const mapDispatchToProps = (dispatch, props) => {
  const { loginChange, login } = bindActionCreators(accountActions, dispatch);
  const { backend } = props;
  return {
    onSettings: () => dispatch(push('/settings')),
    onLogin: (account) => login(backend, account),
    onFirstChangeAfterFailure: () => loginChange({ status: 'none', error: null }),
    onExternalLink: (type) => shell.openExternal(links[type])
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Login);
