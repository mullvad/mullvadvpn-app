import { useEffect, useState } from 'react';
import styled from 'styled-components';

import { strings } from '../../../../../../../../shared/constants';
import {
  EndpointObfuscationType,
  ITunnelEndpoint,
  parseSocketAddress,
  ProxyType,
  RelayProtocol,
  TunnelState,
  TunnelType,
} from '../../../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../../../shared/gettext';
import { colors } from '../../../../../../../lib/foundations';
import { useSelector } from '../../../../../../../redux/store';
import { tinyText } from '../../../../../../common-styles';

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
  color: colors.whiteAlpha60,
});

const StyledConnectionDetailsContainer = styled.div({
  marginTop: '16px',
  marginBottom: '16px',
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
  display: 'block',
  color: colors.white,
  fontWeight: '400',
  minHeight: '1em',
});

const StyledConnectionDetailsTitle = styled(StyledConnectionDetailsLabel)({
  color: colors.whiteAlpha60,
  whiteSpace: 'nowrap',
});

export function ConnectionDetails() {
  const reduxConnection = useSelector((state) => state.connection);
  const [connection, setConnection] = useState(reduxConnection);

  const tunnelState = connection.status;

  useEffect(() => {
    if (
      reduxConnection.status.state === 'connected' ||
      reduxConnection.status.state === 'connecting'
    ) {
      setConnection(reduxConnection);
    }
  }, [reduxConnection, tunnelState.state]);

  const entry = getEntryPoint(tunnelState);

  const showDetails = tunnelState.state === 'connected' || tunnelState.state === 'connecting';
  const hasEntry = showDetails && entry !== undefined;

  return (
    <StyledConnectionDetailsContainer>
      <StyledConnectionDetailsHeading>
        {messages.pgettext('connect-view', 'Connection details')}
      </StyledConnectionDetailsHeading>
      <StyledConnectionDetailsLabel data-testid="tunnel-protocol">
        {showDetails && tunnelState.details !== undefined && strings.wireguard}
      </StyledConnectionDetailsLabel>
      <StyledIpTable>
        <StyledConnectionDetailsTitle>
          {messages.pgettext('connection-info', 'In')}
        </StyledConnectionDetailsTitle>
        <StyledConnectionDetailsLabel data-testid="in-ip">
          {hasEntry ? `${entry.ip}:${entry.port} ${entry.protocol.toUpperCase()}` : ''}
        </StyledConnectionDetailsLabel>
        <StyledConnectionDetailsTitle>
          {messages.pgettext('connection-info', 'Out')}
        </StyledConnectionDetailsTitle>
        <StyledIpLabelContainer>
          {connection.ipv4 && (
            <StyledConnectionDetailsLabel data-testid="out-ip">
              {connection.ipv4}
            </StyledConnectionDetailsLabel>
          )}
          {connection.ipv6 && (
            <StyledConnectionDetailsLabel data-testid="out-ip">
              {connection.ipv6}
            </StyledConnectionDetailsLabel>
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

  const socketAddr = parseSocketAddress(endpoint.obfuscationEndpoint.address);

  return {
    ip: socketAddr.host,
    port: socketAddr.port,
    protocol: endpoint.obfuscationEndpoint.protocol,
    obfuscationType: endpoint.obfuscationEndpoint.obfuscationType,
  };
}
