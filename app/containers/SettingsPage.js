import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import Settings from '../components/Settings';
import userActions from '../actions/user';
import settingsActions from '../actions/settings';

const mapStateToProps = (state) => {
  return state;
};

const mapDispatchToProps = (dispatch, props) => {
  const user = bindActionCreators(userActions, dispatch);
  const settings = bindActionCreators(settingsActions, dispatch);
  return {
    logout: () => {
      return user.logout(props.backend);
    },
    updateSettings: settings.updateSettings
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Settings);
