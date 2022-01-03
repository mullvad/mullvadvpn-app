import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { ITunnelEndpoint, parseSocketAddress } from '../../shared/daemon-rpc-types';
import ConnectionPanel, {
  IBridgeData,
  IInAddress,
  IOutAddress,
} from '../components/ConnectionPanel';
import { IReduxState, ReduxDispatch } from '../redux/store';
import userInterfaceActions from '../redux/userinterface/actions';

function tunnelEndpointToRelayInAddress(tunnelEndpoint: ITunnelEndpoint): IInAddress {
  const socketAddr = parseSocketAddress(tunnelEndpoint.address);
  return {
    ip: socketAddr.host,
    port: socketAddr.port,
    protocol: tunnelEndpoint.protocol,
    tunnelType: tunnelEndpoint.tunnelType,
  };
}

function tunnelEndpointToEntryLocationInAddress(
  tunnelEndpoint: ITunnelEndpoint,
): IInAddress | undefined {
  if (!tunnelEndpoint.entryEndpoint) {
    return undefined;
  }

  const socketAddr = parseSocketAddress(tunnelEndpoint.entryEndpoint.address);
  return {
    ip: socketAddr.host,
    port: socketAddr.port,
    protocol: tunnelEndpoint.entryEndpoint.transportProtocol,
    tunnelType: tunnelEndpoint.tunnelType,
  };
}

function tunnelEndpointToBridgeData(endpoint: ITunnelEndpoint): IBridgeData | undefined {
  if (!endpoint.proxy) {
    return undefined;
  }

  const socketAddr = parseSocketAddress(endpoint.proxy.address);
  return {
    ip: socketAddr.host,
    port: socketAddr.port,
    protocol: endpoint.proxy.protocol,
    bridgeType: endpoint.proxy.proxyType,
  };
}

const mapStateToProps = (state: IReduxState) => {
  const status = state.connection.status;

  const outAddress: IOutAddress = {
    ipv4: state.connection.ipv4,
    ipv6: state.connection.ipv6,
  };

  const inAddress: IInAddress | undefined =
    (status.state === 'connecting' || status.state === 'connected') && status.details
      ? tunnelEndpointToRelayInAddress(status.details.endpoint)
      : undefined;

  const entryLocationInAddress: IInAddress | undefined =
    (status.state === 'connecting' || status.state === 'connected') && status.details
      ? tunnelEndpointToEntryLocationInAddress(status.details.endpoint)
      : undefined;

  const bridgeInfo: IBridgeData | undefined =
    (status.state === 'connecting' || status.state === 'connected') && status.details
      ? tunnelEndpointToBridgeData(status.details.endpoint)
      : undefined;

  return {
    isOpen: state.userInterface.connectionPanelVisible,
    hostname: state.connection.hostname,
    bridgeHostname: state.connection.bridgeHostname,
    entryHostname: state.connection.entryHostname,
    inAddress,
    entryLocationInAddress,
    bridgeInfo,
    outAddress,
  };
};

const mapDispatchToProps = (dispatch: ReduxDispatch) => {
  const userInterface = bindActionCreators(userInterfaceActions, dispatch);

  return {
    onToggle: userInterface.toggleConnectionPanel,
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(ConnectionPanel);
