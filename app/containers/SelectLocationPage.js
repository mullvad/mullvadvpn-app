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
    onSelect: async (host) => {
      try {
        await backend.connect(host);
        dispatch(settingsActions.updateRelay({ host: host }));
        dispatch(push('/connect'));
      } catch (e) {
        log.error('Failed to select server: ', e.message);
      }
    }
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(SelectLocation);
