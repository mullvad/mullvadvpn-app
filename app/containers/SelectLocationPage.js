import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import SelectLocation from '../components/SelectLocation';
import settingsActions from '../actions/settings';

const mapStateToProps = (state) => state;
const mapDispatchToProps = (dispatch, props) => {
  const { backend } = props;
  const settings = bindActionCreators(settingsActions, dispatch);
  return {
    onChangeLocation: (preferredServer) => {
      const server = backend.serverInfo(preferredServer);
      settings.updateSettings({ preferredServer });
      backend.connect(server.address);
    }
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(SelectLocation);
