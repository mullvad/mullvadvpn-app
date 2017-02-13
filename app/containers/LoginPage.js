import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import Login from '../components/Login';
import userActions from '../actions/user';
import { LoginState } from '../constants';

const mapStateToProps = (state) => {
  return state;
};

const mapDispatchToProps = (dispatch, props) => {
  const user = bindActionCreators(userActions, dispatch);
  return {
    onLogin: (account) => {
      return user.requestLogin(props.backend, account);
    },
    onChange: (account) => {
      return user.loginChange({ account });
    },
    onFirstChangeAfterFailure: () => {
      console.log('onFirstChangeAfterFailure');
      return user.loginChange({ status: LoginState.none, error: null })
    }
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Login);
