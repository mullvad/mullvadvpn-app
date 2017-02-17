import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import Connect from '../components/Connect';
import userActions from '../actions/user';

const mapStateToProps = (state) => {
  return state;
};

const mapDispatchToProps = (dispatch) => {
  return {};
};

export default connect(mapStateToProps, mapDispatchToProps)(Connect);
