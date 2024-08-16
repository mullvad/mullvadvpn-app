import styled from 'styled-components';

import { colors } from '../../../config.json';
import {
  ITunnelEndpoint,
  parseSocketAddress,
  ProxyType,
  RelayProtocol,
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

export interface BridgeData extends Endpoint {
  bridgeType: ProxyType;
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

  if (
    (connection.status.state !== 'connecting' && connection.status.state !== 'connected') ||
    connection.status.details?.endpoint === undefined
  ) {
    return null;
  }

  const entry = tunnelEndpointToRelayInAddress(connection.status.details.endpoint);

  return (
    <StyledConnectionDetailsContainer>
      <StyledConnectionDetailsHeading>
        {messages.pgettext('connect-view', 'Connection details')}
      </StyledConnectionDetailsHeading>
      <StyledConnectionDetailsLabel>
        {tunnelTypeToString(entry.tunnelType)}
      </StyledConnectionDetailsLabel>
      <StyledIpTable>
        <StyledConnectionDetailsTitle>
          {messages.pgettext('connection-info', 'In')}
        </StyledConnectionDetailsTitle>
        <StyledConnectionDetailsLabel>
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

function tunnelEndpointToRelayInAddress(tunnelEndpoint: ITunnelEndpoint): InAddress {
  const socketAddr = parseSocketAddress(tunnelEndpoint.address);
  return {
    ip: socketAddr.host,
    port: socketAddr.port,
    protocol: tunnelEndpoint.protocol,
    tunnelType: tunnelEndpoint.tunnelType,
  };
}
