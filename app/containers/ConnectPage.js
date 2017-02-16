import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import Connect from '../components/Connect';
import userActions from '../actions/user';

const mapStateToProps = (state) => {
  return state;
};

const mapDispatchToProps = (dispatch, props) => { // eslint-disable-line no-unused-vars
  const user = bindActionCreators(userActions, dispatch);
  return {
    logout: () => {
      return user.logout(props.backend);
    }
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Connect);
