import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import Settings from '../components/Settings';
import userActions from '../actions/user';

const mapStateToProps = (state) => {
  return state;
};

const mapDispatchToProps = (dispatch, props) => {
  const user = bindActionCreators(userActions, dispatch);
  return {
    logout: () => {
      return user.logout(props.backend);
    }
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Settings);
