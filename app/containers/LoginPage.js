import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import Login from '../components/Login';
import userActions from '../actions/user';
import { LoginState } from '../constants';

const mapStateToProps = (state) => {
  return state;
};

const mapDispatchToProps = (dispatch, props) => {
  const { loginChange, login } = bindActionCreators(userActions, dispatch);
  const { backend } = props;
  return {
    onLogin: (account) => login(backend, account),
    onChange: (account) => loginChange({ account }),
    onFirstChangeAfterFailure: () => loginChange({ status: LoginState.none, error: null })
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Login);
