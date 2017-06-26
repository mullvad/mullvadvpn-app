import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'react-router-redux';
import SelectLocation from '../components/SelectLocation';
import settingsActions from '../redux/settings/actions';

const mapStateToProps = (state) => state;
const mapDispatchToProps = (dispatch, props) => {
  const { backend } = props;
  const settings = bindActionCreators(settingsActions, dispatch);
  return {
    onClose: () => dispatch(push('/connect')),
    onSelect: (preferredServer) => {
      const server = backend.serverInfo(preferredServer);

      dispatch(push('/connect'));

      // add delay to let the map load
      setTimeout(() => {
        settings.updateSettings({ preferredServer });
        backend.connect(server.address);
      }, 600);
    }
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(SelectLocation);
