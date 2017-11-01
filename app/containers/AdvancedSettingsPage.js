import { connect } from 'react-redux';
import { push } from 'react-router-redux';
import { AdvancedSettings } from '../components/AdvancedSettings';
import settingsActions from '../redux/settings/actions';
import log from 'electron-log';

const mapStateToProps = (state) => {
  return {
    protocol: tryOrElse( () => state.settings.relayConstraints.tunnel.openvpn.protocol.only, 'Automatic'),
    port: tryOrElse( () => state.settings.relayConstraints.tunnel.openvpn.port.only, 'Automatic'),
  };
};

function tryOrElse(toTry, orElse) {
  try {
    return toTry() || orElse;
  } catch (e) {
    return orElse;
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

      backend.updateRelayConstraints(update)
        .then( () => dispatch(settingsActions.updateRelay(update)))
        .catch( e => log.error('Failed updating relay constraints', e.message));
    },
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(AdvancedSettings);
