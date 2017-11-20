import { connect } from 'react-redux';
import { push } from 'react-router-redux';
import { AdvancedSettings } from '../components/AdvancedSettings';
import settingsActions from '../redux/settings/actions';
import log from 'electron-log';

const mapStateToProps = (state) => {
  const contraints = state.settings.relaySettings;
  return {
    protocol: anyToAuto(contraints.protocol),
    port: anyToAuto(contraints.port),
  };
};

function anyToAuto(constraint) {
  if (constraint === 'any') {
    return 'Automatic';
  } else {
    return constraint;
  }
}

const mapDispatchToProps = (dispatch, props) => {
  const { backend } = props;
  return {
    onClose: () => dispatch(push('/settings')),

    updateConstraints: (protocol, port) => {

      const protConstraint = protocol === 'Automatic'
        ? 'any'
        : { only: protocol.toLowerCase() };

      const portConstraint = port === 'Automatic'
        ? 'any'
        : { only: port };

      const update = {
        tunnel: { openvpn: {
          protocol: protConstraint,
          port: portConstraint,
        }},
      };

      backend.updateRelaySettings(update)
        .then( () => dispatch(settingsActions.updateRelay({
          port: typeof(portConstraint) === 'object' ? portConstraint.only : portConstraint,
          protocol: typeof(protConstraint) === 'object' ? protConstraint.only : protConstraint,
        })))
        .catch( e => log.error('Failed updating relay constraints', e.message));
    },
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(AdvancedSettings);
