import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { shell } from 'electron';
import { links } from '../config';
import Connect from '../components/Connect';
import connectActions from '../actions/connect';

const mapStateToProps = (state) => {
  return state;
};

const mapDispatchToProps = (dispatch, props) => {
  const { connect, disconnect, copyIPAddress } = bindActionCreators(connectActions, dispatch);
  const { backend } = props;

  return {
    onSettings: () => props.router.push('/settings'),
    onSelectLocation: () => props.router.push('/select-location'),
    onConnect: (addr) => connect(backend, addr),
    onCopyIP: () => copyIPAddress(),
    onDisconnect: () => disconnect(backend),
    onExternalLink: (type) => shell.openExternal(links[type]),
    getServerInfo: (key) => backend.serverInfo(key)
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Connect);
