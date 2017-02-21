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
    onConnect: (addr) => connect(backend, addr),
    onDisconnect: () => disconnect(backend)
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Connect);
