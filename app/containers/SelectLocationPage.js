import { connect } from 'react-redux';
import { push } from 'react-router-redux';
import SelectLocation from '../components/SelectLocation';
import settingsActions from '../redux/settings/actions';
import log from 'electron-log';

const mapStateToProps = (state) => state;
const mapDispatchToProps = (dispatch, props) => {
  const { backend } = props;
  return {
    onClose: () => dispatch(push('/connect')),
    onSelect: (preferredServer) => {

      dispatch(push('/connect'));

      // add delay to let the map load
      setTimeout(() => {
        const update = {
          host: { only: preferredServer },
          tunnel: { openvpn: {
          }},
        };

        backend.updateRelayConstraints(update)
          .then( () => dispatch(settingsActions.updateRelay(update)))
          .then( () => backend.connect())
          .catch( e => log.error('Failed updating relay constraints', e.message));

      }, 600);
    }
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(SelectLocation);
