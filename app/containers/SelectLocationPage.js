import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import SelectLocation from '../components/SelectLocation';
import connectActions from '../actions/connect';
import settingsActions from '../actions/settings';

const mapStateToProps = (state) => {
  return state;
};

const mapDispatchToProps = (dispatch, props) => {
  const user = bindActionCreators(connectActions, dispatch);
  const settings = bindActionCreators(settingsActions, dispatch);
  return {
    updateSettings: settings.updateSettings
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(SelectLocation);
