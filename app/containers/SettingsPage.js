import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import Settings from '../components/Settings';
import userActions from '../actions/user';
import settingsActions from '../actions/settings';

const mapStateToProps = (state) => {
  return state;
};

const mapDispatchToProps = (dispatch, props) => {
  const { logout } = bindActionCreators(userActions, dispatch);
  const { updateSettings } = bindActionCreators(settingsActions, dispatch);
  return {
    logout: () => logout(props.backend),
    updateSettings
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Settings);
