import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import SelectLocation from '../components/SelectLocation';
import settingsActions from '../actions/settings';

const mapStateToProps = (state) => {
  return state;
};

const mapDispatchToProps = (dispatch, props) => {
  return bindActionCreators(settingsActions, dispatch);
};

export default connect(mapStateToProps, mapDispatchToProps)(SelectLocation);
