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

const StyledFlex = styled(Flex)`
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
    <StyledFlex $gap="medium" $padding="medium" $flexDirection="column">
      <AnimateHeight expanded={showDownloadDetails}>
        <DownloadDetails />
      </AnimateHeight>
      {showReportProblemButton ? <ReportProblemButton /> : null}
      {showUpgradeButton ? <UpgradeButton /> : null}
      {showCancelButton ? <CancelButton /> : null}
    </StyledFlex>
  );
};
