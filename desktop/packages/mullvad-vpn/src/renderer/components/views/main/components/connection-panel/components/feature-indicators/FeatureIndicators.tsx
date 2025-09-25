import { useEffect, useRef } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { messages } from '../../../../../../../../shared/gettext';
import { FeatureIndicator, Text } from '../../../../../../../lib/components';
import { colors } from '../../../../../../../lib/foundations';
import { useStyledRef } from '../../../../../../../lib/utility-hooks';
import { useSelector } from '../../../../../../../redux/store';
import { tinyText } from '../../../../../../common-styles';
import { ConnectionPanelAccordion } from '../../../../styles';
import { useGetFeatureIndicator } from './hooks';

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
  color: colors.whiteAlpha60,
});

const StyledFeatureIndicators = styled.div({
  position: 'relative',
});

const StyledFeatureIndicatorsWrapper = styled.div<{ $expanded: boolean }>((props) => ({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'center',
  gap: `${GAP}px`,
  maxHeight: props.$expanded ? 'fit-content' : '56px',
  overflow: 'hidden',
}));

const StyledBaseEllipsis = styled(Text)<{ $display: boolean }>((props) => ({
  position: 'absolute',
  top: `${LINE_HEIGHT + GAP}px`,
  padding: '4px 8px',
  marginLeft: '8px',
  border: '1px solid transparent',
  display: props.$display ? 'inline' : 'none',
}));

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
export function FeatureIndicators(props: FeatureIndicatorsProps) {
  const tunnelState = useSelector((state) => state.connection.status);
  const ellipsisRef = useStyledRef<HTMLSpanElement>();
  const ellipsisSpacerRef = useStyledRef<HTMLSpanElement>();
  const featureIndicatorsContainerRef = useStyledRef<HTMLDivElement>();
  const featureMap = useGetFeatureIndicator();

  const featureIndicatorsVisible =
    tunnelState.state === 'connected' || tunnelState.state === 'connecting';

  const featureIndicators = useRef(
    featureIndicatorsVisible ? (tunnelState.featureIndicators ?? []) : [],
  );

  if (featureIndicatorsVisible && tunnelState.featureIndicators) {
    featureIndicators.current = tunnelState.featureIndicators;
  }

  const ellipsis = messages.gettext('%(amount)d more...');

  useEffect(() => {
    // We need to defer the visibility logic one painting cycle to make sure the elements are
    // rendered and available.
    setTimeout(() => {
      if (
        featureIndicatorsContainerRef.current &&
        ellipsisSpacerRef.current &&
        ellipsisRef.current
      ) {
        // Get all feature indicator elements.
        const indicatorElements = Array.from(
          featureIndicatorsContainerRef.current.getElementsByTagName('button'),
        );

        let lastVisibleIndex = 0;
        let hasHidden = false;
        indicatorElements.forEach((indicatorElement, i) => {
          if (
            indicatorShouldBeVisible(
              props.expanded,
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
            indicatorElement.style.visibility = 'hidden';
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
          // eslint-disable-next-line react-compiler/react-compiler
          ellipsisRef.current.style.left = `${left}px`;
          ellipsisRef.current.style.visibility = 'visible';

          // Add the ellipsis text to the ellipsis.
          ellipsisRef.current.textContent = sprintf(ellipsis, {
            amount: indicatorElements.length - (lastVisibleIndex + 1),
          });
        } else {
          ellipsisRef.current.style.visibility = 'hidden';
        }
      }
    }, 0);
  });

  const sortedIndicators = [...featureIndicators.current].sort((a, b) => a - b);

  return (
    <StyledAccordion expanded={featureIndicatorsVisible && featureIndicators.current.length > 0}>
      <StyledFeatureIndicatorsContainer $expanded={props.expanded}>
        <StyledAccordion expanded={props.expanded}>
          <StyledTitle>{messages.pgettext('connect-view', 'Active features')}</StyledTitle>
        </StyledAccordion>
        <StyledFeatureIndicators>
          <StyledFeatureIndicatorsWrapper
            ref={featureIndicatorsContainerRef}
            $expanded={props.expanded}>
            {sortedIndicators.map((indicator) => {
              const feature = featureMap[indicator];
              return (
                <FeatureIndicator
                  key={indicator.toString()}
                  data-testid="feature-indicator"
                  onClick={feature.onClick}>
                  <FeatureIndicator.Text>{feature.label}</FeatureIndicator.Text>
                </FeatureIndicator>
              );
            })}
          </StyledFeatureIndicatorsWrapper>
          <StyledEllipsisSpacer
            variant="labelTinySemiBold"
            $display={!props.expanded}
            ref={ellipsisSpacerRef}>
            {
              // Mock amount for the spacer ellipsis. This needs to be wider than the real
              // ellipsis will ever be.
              sprintf(ellipsis, { amount: 222 })
            }
          </StyledEllipsisSpacer>
          <StyledEllipsis
            variant="labelTinySemiBold"
            onClick={props.expandIsland}
            $display={!props.expanded}
            ref={ellipsisRef}
          />
        </StyledFeatureIndicators>
      </StyledFeatureIndicatorsContainer>
    </StyledAccordion>
  );
}

function indicatorShouldBeVisible(
  expanded: boolean,
  container: HTMLElement,
  indicator: HTMLElement,
  ellipsisSpacer: HTMLElement,
): boolean {
  if (expanded) {
    return true;
  }

  const indicatorRect = indicator.getBoundingClientRect();
  const ellipsisSpacerRect = ellipsisSpacer.getBoundingClientRect();
  const containerRect = container.getBoundingClientRect();

  // Calculate which line the indicator is positioned on.
  const lineIndex = Math.round((indicatorRect.top - containerRect.top) / (LINE_HEIGHT + GAP));

  // An indicator should be visible if it's on the first line or if it is on the second line and
  // doesn't overlap with the ellipsis.
  return lineIndex === 0 || (lineIndex === 1 && indicatorRect.right < ellipsisSpacerRect.left);
}
