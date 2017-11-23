import { connect } from 'react-redux';
import { push } from 'react-router-redux';
import { AdvancedSettings } from '../components/AdvancedSettings';
import settingsActions from '../redux/settings/actions';
import log from 'electron-log';

const mapStateToProps = (state) => {
  const constraints = state.settings.relaySettings;
  const { host, protocol, port } = constraints;
  return {
    host: host,
    protocol: protocol === 'any' ? 'Automatic' : protocol,
    port: port === 'any' ? 'Automatic' : port,
  };
};

const mapDispatchToProps = (dispatch, props) => {
  const { backend } = props;
  return {
    onClose: () => dispatch(push('/settings')),

    onUpdateConstraints: (host, protocol, port) => {
      // TODO: udp and 1301 are automatic because we cannot pass `any` when using custom tunnel
      const protocolConstraint = protocol === 'Automatic' ? 'udp' : protocol.toLowerCase();
      const portConstraint = port === 'Automatic' ? (protocolConstraint === 'tcp' ? 443 : 1301) : port;
      const update = {
        custom_tunnel_endpoint: {
          host: host,
          tunnel: {
            openvpn: {
              protocol: protocolConstraint,
              port: portConstraint,
            }
          }
        },
      };

      backend.updateRelaySettings(update)
        .then( () => dispatch(settingsActions.updateRelay({
          protocol: protocolConstraint,
          port: portConstraint,
        })))
        .catch( e => log.error('Failed updating relay constraints', e.message));
    },
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(AdvancedSettings);
