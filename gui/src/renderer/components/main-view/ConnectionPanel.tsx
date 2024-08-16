import { useEffect } from 'react';
import styled from 'styled-components';

import { useBoolean } from '../../lib/utilityHooks';
import { useSelector } from '../../redux/store';
import Accordion from '../Accordion';
import CustomScrollbars from '../CustomScrollbars';
import ConnectionActionButton from './ConnectionActionButton';
import ConnectionDetails from './ConnectionDetails';
import ConnectionPanelChevron from './ConnectionPanelChevron';
import ConnectionStatus from './ConnectionStatus';
import FeatureIndicators from './FeatureIndicators';
import Hostname from './Hostname';
import Location from './Location';
import SelectLocationButton from './SelectLocationButton';

const PANEL_MARGIN = '16px';

const StyledAccordion = styled(Accordion)({
  flexShrink: 0,
});

const StyledConnectionPanel = styled.div<{ $expanded: boolean }>((props) => ({
  position: 'relative',
  display: 'flex',
  flexDirection: 'column',
  maxHeight: `calc(100% - 2 * ${PANEL_MARGIN})`,
  margin: `auto ${PANEL_MARGIN} ${PANEL_MARGIN}`,
  padding: '16px',
  justifySelf: 'flex-end',
  borderRadius: '12px',
  backgroundColor: `rgba(16, 24, 35, ${props.$expanded ? 0.8 : 0.4})`,
  backdropFilter: 'blur(6px)',

  transition: 'background-color 250ms ease-out',
}));

const StyledConnectionButtonContainer = styled.div<{ $showMargin: boolean }>((props) => ({
  transition: 'margin-top 250ms ease-out',
  display: 'flex',
  flexDirection: 'column',
  gap: '16px',
  marginTop: props.$showMargin ? '16px' : 0,
}));

const StyledCustomScrollbars = styled(CustomScrollbars)({
  flexShrink: 1,
});

const StyledConnectionPanelChevron = styled(ConnectionPanelChevron)({
  position: 'absolute',
  top: '16px',
  right: '16px',
  width: 'fit-content',
});

const StyledConnectionStatusContainer = styled.div<{ $expanded: boolean; $showMargin: boolean }>(
  (props) => ({
    paddingBottom: '16px',
    marginBottom: props.$expanded && props.$showMargin ? '16px' : 0,
    borderBottom: props.$expanded ? '1px rgba(255, 255, 255, 0.2) solid' : 'none',
    transitionProperty: 'margin-bottom, padding-bottom',
    transitionDuration: '250ms',
    transitionTimingFunction: 'ease-out',
  }),
);

export default function ConnectionPanel() {
  const [expanded, expand, collapse, toggleExpanded] = useBoolean();
  const tunnelState = useSelector((state) => state.connection.status);

  const allowExpand = tunnelState.state === 'connected' || tunnelState.state === 'connecting';
  const hasFeatureIndicators =
    allowExpand &&
    tunnelState.featureIndicators !== undefined &&
    tunnelState.featureIndicators.length > 0;

  useEffect(collapse, [tunnelState, collapse]);

  return (
    <StyledConnectionPanel $expanded={expanded}>
      {allowExpand && (
        <StyledConnectionPanelChevron pointsUp={!expanded} onToggle={toggleExpanded} />
      )}
      <StyledConnectionStatusContainer
        $expanded={expanded}
        $showMargin={hasFeatureIndicators}
        onClick={toggleExpanded}>
        <ConnectionStatus />
        <Location />
        <Hostname />
      </StyledConnectionStatusContainer>
      <StyledCustomScrollbars>
        <FeatureIndicators expanded={expanded} expandIsland={expand} />
        <StyledAccordion expanded={expanded}>
          <ConnectionDetails />
        </StyledAccordion>
      </StyledCustomScrollbars>
      <StyledConnectionButtonContainer $showMargin={hasFeatureIndicators}>
        <SelectLocationButton />
        <ConnectionActionButton />
      </StyledConnectionButtonContainer>
    </StyledConnectionPanel>
  );
}
