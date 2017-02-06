import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import Tray from '../components/Tray';
import userActions from '../actions/user';

const mapStateToProps = (state) => {
  return state;
};

const mapDispatchToProps = (dispatch) => {
  return bindActionCreators(userActions, dispatch);
};

export default connect(mapStateToProps, mapDispatchToProps)(Tray);
