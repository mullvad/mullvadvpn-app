import styled from 'styled-components';

import { TunnelState } from '../../../shared/daemon-rpc-types';
import { Colors } from '../../lib/foundations';
import { useSelector } from '../../redux/store';
import { largeText } from '../common-styles';
import Marquee from '../Marquee';
import { ConnectionPanelAccordion } from './styles';

const StyledLocation = styled.span(largeText, {
  color: Colors.white,
  flexShrink: 0,
});

export default function Location() {
  const connection = useSelector((state) => state.connection);
  const text = getLocationText(connection.status, connection.country, connection.city);

  return (
    <ConnectionPanelAccordion expanded={connection.status.state !== 'error'}>
      <StyledLocation>
        <Marquee>{text}</Marquee>
      </StyledLocation>
    </ConnectionPanelAccordion>
  );
}

function getLocationText(tunnelState: TunnelState, country?: string, city?: string): string {
  country = country ?? '';

  switch (tunnelState.state) {
    case 'connected':
    case 'connecting':
      return city ? `${country}, ${city}` : country;
    case 'disconnecting':
    case 'disconnected':
      return country;
    case 'error':
      return '';
  }
}
