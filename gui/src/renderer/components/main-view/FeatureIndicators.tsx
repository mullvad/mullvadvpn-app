import { useLayoutEffect, useRef } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { colors, strings } from '../../../config.json';
import { FeatureIndicator } from '../../../shared/daemon-rpc-types';
import { messages } from '../../../shared/gettext';
import { useStyledRef } from '../../lib/utilityHooks';
import { useSelector } from '../../redux/store';
import { tinyText } from '../common-styles';
import { ConnectionPanelAccordion } from './styles';

const LINE_HEIGHT = 22;
const GAP = 8;

const StyledAccordion = styled(ConnectionPanelAccordion)({
  flexShrink: 0,
});

const StyledFeatureIndicatorsContainer = styled.div<{ $expanded: boolean }>((props) => ({
  marginTop: '0px',
  marginBottom: props.$expanded ? '8px' : 0,
  transition: 'margin-bottom 300ms ease-out',
}));

const StyledTitle = styled.h2(tinyText, {
  margin: '0 0 2px',
  fontSize: '10px',
  lineHeight: '15px',
  color: colors.white60,
});

const StyledFeatureIndicators = styled.div({
  position: 'relative',
});

const StyledFeatureIndicatorsWrapper = styled.div<{ $expanded: boolean }>((props) => ({
  display: 'flex',
  flexWrap: 'wrap',
  gap: `${GAP}px`,
  maxHeight: props.$expanded ? 'fit-content' : '52px',
  overflow: 'hidden',
}));

const StyledFeatureIndicatorLabel = styled.span<{ $expanded: boolean }>(tinyText, (props) => ({
  display: 'inline',
  padding: '2px 8px',
  justifyContent: 'center',
  alignItems: 'center',
  borderRadius: '4px',
  background: colors.darkerBlue,
  color: colors.white,
  fontWeight: 400,
  whiteSpace: 'nowrap',
  visibility: props.$expanded ? 'visible' : 'hidden',
}));

const StyledBaseEllipsis = styled.span(tinyText, {
  position: 'absolute',
  top: `${LINE_HEIGHT + GAP}px`,
  color: colors.white,
  padding: '2px 8px 2px 16px',
});

const StyledEllipsisSpacer = styled(StyledBaseEllipsis)({
  right: 0,
  opacity: 0,
});

const StyledEllipsis = styled(StyledBaseEllipsis)({
  visibility: 'hidden',
});

interface FeatureIndicatorsProps {
  expanded: boolean;
  expandIsland: () => void;
}

// This component needs to render a maximum of two lines of feature indicators and then ellipsis
// with the text "N more...". This poses two challenges:
// 1. We can't know the size of the content beforehand or how many indicators should be hidden
// 2. The ellipsis string doesn't have a fixed width, the amount can change.
//
// To solve this the indicators are first rendered hidden along with a invisible "placeholder"
// ellipsis at the end of the second row. Then after render, all indicators that either is placed
// after the second row or overlaps with the invisible ellipsis text will be set to invisible. Then
// we can count those and add another ellipsis element which is visible and place it after the last
// visible indicator.
export default function FeatureIndicators(props: FeatureIndicatorsProps) {
  const tunnelState = useSelector((state) => state.connection.status);
  const ellipsisRef = useStyledRef<HTMLSpanElement>();
  const ellipsisSpacerRef = useStyledRef<HTMLSpanElement>();
  const featureIndicatorsContainerRef = useStyledRef<HTMLDivElement>();

  const featureIndicatorsVisible =
    tunnelState.state === 'connected' || tunnelState.state === 'connecting';
  const featureIndicators = useRef(
    featureIndicatorsVisible ? tunnelState.featureIndicators ?? [] : [],
  );

  if (featureIndicatorsVisible && tunnelState.featureIndicators) {
    featureIndicators.current = tunnelState.featureIndicators;
  }

  const ellipsis = messages.gettext('%(amount)d more...');

  useLayoutEffect(() => {
    if (
      !props.expanded &&
      featureIndicatorsContainerRef.current &&
      ellipsisSpacerRef.current &&
      ellipsisRef.current
    ) {
      // Get all feature indicator elements.
      const indicatorElements = Array.from(
        featureIndicatorsContainerRef.current.getElementsByTagName('span'),
      );
      let lastVisibleIndex = 0;
      let hasHidden = false;
      indicatorElements.forEach((indicatorElement, i) => {
        if (
          !indicatorShouldBeHidden(
            featureIndicatorsContainerRef.current!,
            indicatorElement,
            ellipsisSpacerRef.current!,
          )
        ) {
          // If an indicator should be visible we set its visibility and increment the variable
          // containing the last visible index.
          indicatorElement.style.visibility = 'visible';
          lastVisibleIndex = i;
        } else {
          // If it should be visible we store that there exists hidden indicators.
          hasHidden = true;
        }
      });

      if (hasHidden) {
        const lastVisibleIndicatorRect =
          indicatorElements[lastVisibleIndex].getBoundingClientRect();
        const containerRect = featureIndicatorsContainerRef.current.getBoundingClientRect();

        // Place the ellipsis at the end of the last visible indicator.
        const left = lastVisibleIndicatorRect.right - containerRect.left;
        ellipsisRef.current.style.left = `${left}px`;
        ellipsisRef.current.style.visibility = 'visible';

        // Add the ellipsis text to the ellipsis.
        ellipsisRef.current.textContent = sprintf(ellipsis, {
          amount: indicatorElements.length - (lastVisibleIndex + 1),
        });
      }
    }
  });

  return (
    <StyledAccordion expanded={featureIndicatorsVisible && featureIndicators.current.length > 0}>
      <StyledFeatureIndicatorsContainer onClick={props.expandIsland} $expanded={props.expanded}>
        <StyledAccordion expanded={props.expanded}>
          <StyledTitle>{messages.pgettext('connect-view', 'Active features')}</StyledTitle>
        </StyledAccordion>
        <StyledFeatureIndicators>
          <StyledFeatureIndicatorsWrapper
            ref={featureIndicatorsContainerRef}
            $expanded={props.expanded}>
            {featureIndicators.current.sort().map((indicator) => (
              <StyledFeatureIndicatorLabel
                key={indicator.toString()}
                data-testid="feature-indicator"
                $expanded={props.expanded}>
                {getFeatureIndicatorLabel(indicator)}
              </StyledFeatureIndicatorLabel>
            ))}
          </StyledFeatureIndicatorsWrapper>
          {!props.expanded && (
            <>
              <StyledEllipsis ref={ellipsisRef} />
              <StyledEllipsisSpacer ref={ellipsisSpacerRef}>
                {
                  // Mock amount for the spacer ellipsis. This needs to be wider than the real
                  // ellipsis will ever be.
                  sprintf(ellipsis, { amount: 222 })
                }
              </StyledEllipsisSpacer>
            </>
          )}
        </StyledFeatureIndicators>
      </StyledFeatureIndicatorsContainer>
    </StyledAccordion>
  );
}

function indicatorShouldBeHidden(
  container: HTMLElement,
  indicator: HTMLElement,
  ellipsisSpacer: HTMLElement,
): boolean {
  const indicatorRect = indicator.getBoundingClientRect();
  const ellipsisSpacerRect = ellipsisSpacer.getBoundingClientRect();
  const containerRect = container.getBoundingClientRect();

  // Calculate which line the indicator is positioned on.
  const lineIndex = Math.round((indicatorRect.top - containerRect.top) / (LINE_HEIGHT + GAP));

  return lineIndex > 1 || (lineIndex === 1 && indicatorRect.right >= ellipsisSpacerRect.left);
}

function getFeatureIndicatorLabel(indicator: FeatureIndicator) {
  switch (indicator) {
    case FeatureIndicator.daita:
      return strings.daita;
    case FeatureIndicator.udp2tcp:
    case FeatureIndicator.shadowsocks:
      return messages.pgettext('wireguard-settings-view', 'Obfuscation');
    case FeatureIndicator.multihop:
      // TRANSLATORS: This refers to the multihop setting in the VPN settings view. This is
      // TRANSLATORS: displayed when the feature is on.
      return messages.gettext('Multihop');
    case FeatureIndicator.customDns:
      // TRANSLATORS: This refers to the Custom DNS setting in the VPN settings view. This is
      // TRANSLATORS: displayed when the feature is on.
      return messages.gettext('Custom DNS');
    case FeatureIndicator.customMtu:
      return messages.pgettext('wireguard-settings-view', 'MTU');
    case FeatureIndicator.bridgeMode:
      return messages.pgettext('openvpn-settings-view', 'Bridge mode');
    case FeatureIndicator.lanSharing:
      return messages.pgettext('vpn-settings-view', 'Local network sharing');
    case FeatureIndicator.customMssFix:
      return messages.pgettext('openvpn-settings-view', 'Mssfix');
    case FeatureIndicator.lockdownMode:
      return messages.pgettext('vpn-settings-view', 'Lockdown mode');
    case FeatureIndicator.splitTunneling:
      return strings.splitTunneling;
    case FeatureIndicator.serverIpOverride:
      return messages.pgettext('settings-import', 'Server IP override');
    case FeatureIndicator.quantumResistance:
      // TRANSLATORS: This refers to the quantum resistance setting in the WireGuard settings view.
      // TRANSLATORS: This is displayed when the feature is on.
      return messages.gettext('Quantum resistance');
    case FeatureIndicator.dnsContentBlockers:
      return messages.pgettext('vpn-settings-view', 'DNS content blockers');
  }
}
