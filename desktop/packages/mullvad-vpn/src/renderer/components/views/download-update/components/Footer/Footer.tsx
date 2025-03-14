import styled from 'styled-components';

import { Flex } from '../../../../../lib/components';
import { AnimateHeight } from '../../../../AnimateHeight';
import { CancelButton, DownloadDetails, ReportProblemButton, UpgradeButton } from './components';
import {
  useShowCancelButton,
  useShowDownloadDetails,
  useShowReportProblemButton,
  useShowUpgradeButton,
} from './hooks';

const StyledFooter = styled.div`
  // TODO: Use color from Colors
  background-color: rgba(21, 39, 58, 1);
  position: sticky;
  bottom: 0;
  width: 100%;
`;

export const Footer = () => {
  const showCancelButton = useShowCancelButton();
  const showDownloadDetails = useShowDownloadDetails();
  const showUpgradeButton = useShowUpgradeButton();
  const showReportProblemButton = useShowReportProblemButton();

  return (
    <StyledFooter>
      <Flex $padding="large" $flexDirection="column">
        <AnimateHeight expanded={showDownloadDetails}>
          {/* TODO: The AnimateHeight component doesn't work very well together
              with the Flex $gap prop, as when the AnimateHeight is collapsed
              and doesn't show any content, it unfortunately adds an extra "gap"
              which is only desired when expanded.
          */}
          <Flex $flexDirection="column" $margin={{ bottom: 'medium' }}>
            <DownloadDetails />
          </Flex>
        </AnimateHeight>
        <Flex $gap="medium" $flexDirection="column">
          {showReportProblemButton ? <ReportProblemButton /> : null}
          {showUpgradeButton ? <UpgradeButton /> : null}
          {showCancelButton ? <CancelButton /> : null}
        </Flex>
      </Flex>
    </StyledFooter>
  );
};
