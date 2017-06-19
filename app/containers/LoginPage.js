import { shell } from 'electron';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'react-router-redux';
import Login from '../components/Login';
import userActions from '../actions/user';
import { links } from '../config';

const mapStateToProps = (state) => state;
const mapDispatchToProps = (dispatch, props) => {
  const { loginChange, login } = bindActionCreators(userActions, dispatch);
  const { backend } = props;
  return {
    onSettings: () => dispatch(push('/settings')),
    onLogin: (account) => login(backend, account),
    onChange: (account) => loginChange({ account }),
    onFirstChangeAfterFailure: () => loginChange({ status: 'none', error: null }),
    onExternalLink: (type) => shell.openExternal(links[type])
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Login);
