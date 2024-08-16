import styled from 'styled-components';

import { colors } from '../../../config.json';
import {
  EndpointObfuscationType,
  ITunnelEndpoint,
  parseSocketAddress,
  ProxyType,
  RelayProtocol,
  TunnelState,
  TunnelType,
  tunnelTypeToString,
} from '../../../shared/daemon-rpc-types';
import { messages } from '../../../shared/gettext';
import { useSelector } from '../../redux/store';
import { tinyText } from '../common-styles';

interface Endpoint {
  ip: string;
  port: number;
  protocol: RelayProtocol;
}

interface InAddress extends Endpoint {
  tunnelType: TunnelType;
}

interface BridgeData extends Endpoint {
  bridgeType: ProxyType;
}

interface ObfuscationData extends Endpoint {
  obfuscationType: EndpointObfuscationType;
}

const StyledConnectionDetailsHeading = styled.h2(tinyText, {
  margin: '0 0 4px',
  fontSize: '10px',
  lineHeight: '15px',
  color: colors.white60,
});

const StyledConnectionDetailsContainer = styled.div({
  marginTop: '24px',
  marginBottom: '18px',
});

const StyledIpTable = styled.div({
  display: 'grid',
  gridTemplateColumns: 'minmax(48px, min-content) auto',
});

const StyledIpLabelContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
});

const StyledConnectionDetailsLabel = styled.span(tinyText, {
  color: colors.white,
  fontWeight: '400',
});

const StyledConnectionDetailsTitle = styled(StyledConnectionDetailsLabel)({
  color: colors.white60,
  whiteSpace: 'nowrap',
});

export default function ConnectionDetails() {
  const connection = useSelector((state) => state.connection);
  const tunnelState = connection.status;
  const entry = getEntryPoint(tunnelState);

  if (
    (tunnelState.state !== 'connected' && tunnelState.state !== 'connecting') ||
    entry === undefined
  ) {
    return null;
  }

  return (
    <StyledConnectionDetailsContainer>
      <StyledConnectionDetailsHeading>
        {messages.pgettext('connect-view', 'Connection details')}
      </StyledConnectionDetailsHeading>
      <StyledConnectionDetailsLabel data-testid="tunnel-protocol">
        {tunnelState.details && tunnelTypeToString(tunnelState.details?.endpoint.tunnelType)}
      </StyledConnectionDetailsLabel>
      <StyledIpTable>
        <StyledConnectionDetailsTitle>
          {messages.pgettext('connection-info', 'In')}
        </StyledConnectionDetailsTitle>
        <StyledConnectionDetailsLabel data-testid="in-ip">
          {`${entry.ip}:${entry.port} ${entry.protocol.toUpperCase()}`}
        </StyledConnectionDetailsLabel>
        <StyledConnectionDetailsTitle>
          {messages.pgettext('connection-info', 'Out')}
        </StyledConnectionDetailsTitle>
        <StyledIpLabelContainer>
          {connection.ipv4 && (
            <StyledConnectionDetailsLabel>{connection.ipv4}</StyledConnectionDetailsLabel>
          )}
          {connection.ipv6 && (
            <StyledConnectionDetailsLabel>{connection.ipv6}</StyledConnectionDetailsLabel>
          )}
        </StyledIpLabelContainer>
      </StyledIpTable>
    </StyledConnectionDetailsContainer>
  );
}

function getEntryPoint(tunnelState: TunnelState): Endpoint | undefined {
  if (
    (tunnelState.state !== 'connected' && tunnelState.state !== 'connecting') ||
    tunnelState.details === undefined
  ) {
    return undefined;
  }

  const endpoint = tunnelState.details.endpoint;
  const inAddress = tunnelEndpointToRelayInAddress(endpoint);
  const entryLocationInAddress = tunnelEndpointToEntryLocationInAddress(endpoint);
  const bridgeInfo = tunnelEndpointToBridgeData(endpoint);
  const obfuscationEndpoint = tunnelEndpointToObfuscationEndpoint(endpoint);

  if (obfuscationEndpoint) {
    return obfuscationEndpoint;
  } else if (entryLocationInAddress && inAddress) {
    return entryLocationInAddress;
  } else if (bridgeInfo && inAddress) {
    return bridgeInfo;
  } else {
    return inAddress;
  }
}

function tunnelEndpointToRelayInAddress(tunnelEndpoint: ITunnelEndpoint): InAddress {
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
): InAddress | undefined {
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

function tunnelEndpointToBridgeData(endpoint: ITunnelEndpoint): BridgeData | undefined {
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

function tunnelEndpointToObfuscationEndpoint(
  endpoint: ITunnelEndpoint,
): ObfuscationData | undefined {
  if (!endpoint.obfuscationEndpoint) {
    return undefined;
  }

  return {
    ip: endpoint.obfuscationEndpoint.address,
    port: endpoint.obfuscationEndpoint.port,
    protocol: endpoint.obfuscationEndpoint.protocol,
    obfuscationType: endpoint.obfuscationEndpoint.obfuscationType,
  };
}
