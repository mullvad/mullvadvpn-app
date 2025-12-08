import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { messages } from '../../../../../../../../shared/gettext';
import { colors } from '../../../../../../../lib/foundations';
import { IConnectionReduxState } from '../../../../../../../redux/connection/reducers';
import { useSelector } from '../../../../../../../redux/store';
import { smallText } from '../../../../../../common-styles';
import Marquee from '../../../../../../Marquee';
import { ConnectionPanelAccordion } from '../../../../styles';

const StyledAccordion = styled(ConnectionPanelAccordion)({
  flexShrink: 0,
});

const StyledHostname = styled.span(smallText, {
  color: colors.whiteAlpha60,
  fontWeight: '400',
  flexShrink: 0,
  minHeight: '1em',
});

export function Hostname() {
  const tunnelState = useSelector((state) => state.connection.status.state);
  const connection = useSelector((state) => state.connection);
  const text = getHostnameText(connection);

  return (
    <StyledAccordion expanded={tunnelState === 'connecting' || tunnelState === 'connected'}>
      <StyledHostname data-testid="hostname-line">
        <Marquee>{text}</Marquee>
      </StyledHostname>
    </StyledAccordion>
  );
}

function getHostnameText(connection: IConnectionReduxState) {
  let hostname = '';
  if (connection.hostname && connection.entryHostname) {
    hostname = sprintf(
      // TRANSLATORS: The hostname line displayed below the country on the main screen
      // TRANSLATORS: Available placeholders:
      // TRANSLATORS: %(relay)s - the relay hostname
      // TRANSLATORS: %(entry)s - the entry relay hostname
      messages.pgettext('connection-info', '%(relay)s via %(entry)s'),
      {
        relay: connection.hostname,
        entry: connection.entryHostname,
      },
    );
  } else if (
    (connection.status.state === 'connecting' || connection.status.state === 'connected') &&
    connection.status.details?.endpoint.proxy !== undefined
  ) {
    hostname = sprintf(messages.pgettext('connection-info', '%(relay)s via Custom bridge'), {
      relay: connection.hostname,
    });
  } else if (connection.hostname) {
    hostname = connection.hostname;
  }

  return hostname;
}
