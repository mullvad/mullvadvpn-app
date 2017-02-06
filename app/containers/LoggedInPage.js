import { connect } from 'react-redux';
import LoggedIn from '../components/LoggedIn';

const mapStateToProps = (state) => {
  return state;
};

const mapDispatchToProps = (dispatch) => { // eslint-disable-line no-unused-vars
  return {};
};

export default connect(mapStateToProps, mapDispatchToProps)(LoggedIn);
