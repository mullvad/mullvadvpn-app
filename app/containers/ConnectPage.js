import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import Connect from '../components/Connect';
import connectActions from '../actions/connect';

const mapStateToProps = (state) => {
  return state;
};

const mapDispatchToProps = (dispatch, props) => {
  const { connect, disconnect } = bindActionCreators(connectActions, dispatch);
  const { backend } = props;
  return {
    onSettings: () => props.router.push('/settings'),
    onSelectLocation: () => props.router.push('/select-location'),
    onConnect: (addr) => connect(backend, addr),
    onDisconnect: () => disconnect(backend),
    getServerInfo: (key) => backend.serverInfo(key)
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Connect);
