import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import SelectLocation from '../components/SelectLocation';
import settingsActions from '../actions/settings';

const mapStateToProps = (state) => state;
const mapDispatchToProps = (dispatch, props) => {
  const { backend } = props;
  const settings = bindActionCreators(settingsActions, dispatch);
  return {
    onClose: () => props.router.push('/connect'),
    onSelect: (preferredServer) => {
      const server = backend.serverInfo(preferredServer);

      props.router.push('/connect');
      
      // add delay to let the map load
      setTimeout(() => {
        settings.updateSettings({ preferredServer });
        backend.connect(server.address);
      }, 600);
    }
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(SelectLocation);
