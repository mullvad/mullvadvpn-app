import { useCallback, useEffect } from 'react';
import styled from 'styled-components';

import { IconButton } from '../../../../../lib/components';
import { colors } from '../../../../../lib/foundations';
import { useBoolean } from '../../../../../lib/utility-hooks';
import { useSelector } from '../../../../../redux/store';
import CustomScrollbars from '../../../../CustomScrollbars';
import { BackAction } from '../../../../KeyboardNavigation';
import { ConnectionPanelAccordion } from '../../styles';
import {
  ConnectionActionButton,
  ConnectionDetails,
  ConnectionStatus,
  FeatureIndicators,
  Hostname,
  Location,
} from './components';
import SelectLocationButton from './components/select-location-button/SelectLocationButton';

const PANEL_MARGIN = '16px';

const StyledAccordion = styled(ConnectionPanelAccordion)({
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
  backgroundColor: props.$expanded ? colors.darkerBlue10Alpha80 : colors.darkerBlue10Alpha40,
  backdropFilter: 'blur(6px)',

  transition: 'background-color 300ms ease-out',
}));

const StyledConnectionButtonContainer = styled.div({
  transition: 'margin-top 300ms ease-out',
  display: 'flex',
  flexDirection: 'column',
  gap: '16px',
  marginTop: '16px',
});

const StyledCustomScrollbars = styled(CustomScrollbars)({
  flexShrink: 1,
});

const StyledConnectionPanelChevron = styled(IconButton)({
  position: 'absolute',
  top: '16px',
  right: '16px',
  width: 'fit-content',
});

const StyledConnectionStatusContainer = styled.div<{
  $expanded: boolean;
  $hasFeatureIndicators: boolean;
}>((props) => ({
  paddingBottom: props.$hasFeatureIndicators || props.$expanded ? '16px' : 0,
  marginBottom: props.$expanded && props.$hasFeatureIndicators ? '16px' : 0,
  borderBottom: props.$expanded ? `1px ${colors.whiteAlpha20} solid` : 'none',
  transitionProperty: 'margin-bottom, padding-bottom',
  transitionDuration: '300ms',
  transitionTimingFunction: 'ease-out',
}));

export function ConnectionPanel() {
  const [expanded, expandImpl, collapse, toggleExpandedImpl] = useBoolean();
  const tunnelState = useSelector((state) => state.connection.status);

  const allowExpand = tunnelState.state === 'connected' || tunnelState.state === 'connecting';

  const expand = useCallback(() => {
    if (allowExpand) {
      expandImpl();
    }
  }, [allowExpand, expandImpl]);

  const toggleExpanded = useCallback(() => {
    if (allowExpand) {
      toggleExpandedImpl();
    }
  }, [allowExpand, toggleExpandedImpl]);

  const hasFeatureIndicators =
    allowExpand &&
    tunnelState.featureIndicators !== undefined &&
    tunnelState.featureIndicators.length > 0;

  useEffect(collapse, [tunnelState.state, collapse]);

  return (
    <BackAction disabled={!expanded} action={collapse}>
      <StyledConnectionPanel $expanded={expanded}>
        {allowExpand && (
          <StyledConnectionPanelChevron
            onClick={toggleExpanded}
            data-testid="connection-panel-chevron">
            <IconButton.Icon icon={expanded ? 'chevron-down' : 'chevron-up'} />
          </StyledConnectionPanelChevron>
        )}
        <StyledConnectionStatusContainer
          $expanded={expanded}
          $hasFeatureIndicators={hasFeatureIndicators}
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
        <StyledConnectionButtonContainer>
          <SelectLocationButton />
          <ConnectionActionButton />
        </StyledConnectionButtonContainer>
      </StyledConnectionPanel>
    </BackAction>
  );
}
