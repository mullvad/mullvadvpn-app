import styled from 'styled-components';

import { colors } from '../../../config.json';
import { TunnelState } from '../../../shared/daemon-rpc-types';
import { useSelector } from '../../redux/store';
import Accordion from '../Accordion';
import { largeText } from '../common-styles';
import Marquee from '../Marquee';

const StyledLocation = styled.span(largeText, {
  color: colors.white,
  flexShrink: 0,
});

export default function Location() {
  const connection = useSelector((state) => state.connection);
  const text = getLocationText(connection.status, connection.country, connection.city);

  return (
    <Accordion expanded={connection.status.state !== 'error'}>
      <StyledLocation>
        <Marquee>{text}</Marquee>
      </StyledLocation>
    </Accordion>
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
