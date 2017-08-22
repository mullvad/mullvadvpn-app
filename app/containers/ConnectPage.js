import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'react-router-redux';
import { shell } from 'electron';
import { links } from '../config';
import Connect from '../components/Connect';
import connectActions from '../redux/connection/actions';

const mapStateToProps = (state) => {
  return {
    accountPaidUntil: state.account.paidUntil,
    connection: state.connection,
    preferredServer: state.settings.preferredServer,
  };
};

const mapDispatchToProps = (dispatch, props) => {
  const { connect, disconnect, copyIPAddress } = bindActionCreators(connectActions, dispatch);
  const { backend } = props;

  return {
    onSettings: () => dispatch(push('/settings')),
    onSelectLocation: () => dispatch(push('/select-location')),
    onConnect: (addr) => connect(backend, addr),
    onCopyIP: () => copyIPAddress(),
    onDisconnect: () => disconnect(backend),
    onExternalLink: (type) => shell.openExternal(links[type]),
    getServerInfo: (key) => backend.serverInfo(key)
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Connect);
