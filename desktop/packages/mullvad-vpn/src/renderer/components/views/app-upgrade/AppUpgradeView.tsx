import styled from 'styled-components';

import { Flex } from '../../../lib/components';
import { useHistory } from '../../../lib/history';
import { AnimateHeight } from '../../AnimateHeight';
import { BackAction } from '../../KeyboardNavigation';
import { Layout } from '../../Layout';
import {
  CancelButton,
  DownloadLabel,
  DownloadProgress,
  ReportProblemButton,
  UpgradeButton,
  UpgradeDetails,
} from './components';
import {
  useShowCancelButton,
  useShowDownloadDetails,
  useShowDownloadLabel,
  useShowDownloadProgress,
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

export const AppUpgradeView = () => {
  const { pop } = useHistory();
  const showCancelButton = useShowCancelButton();
  const showDownloadDetails = useShowDownloadDetails();
  const showDownloadLabel = useShowDownloadLabel();
  const showDownloadProgress = useShowDownloadProgress();
  const showReportProblemButton = useShowReportProblemButton();
  const showUpgradeButton = useShowUpgradeButton();

  return (
    <BackAction action={pop}>
      <Layout>
        <UpgradeDetails />
        <StyledFooter>
          <Flex $padding="large" $flexDirection="column">
            <AnimateHeight expanded={showDownloadDetails}>
              {/* TODO: The AnimateHeight component doesn't work very well together
               * with the Flex $gap prop, as when the AnimateHeight is collapsed
               * and doesn't show any content, it unfortunately adds an extra "gap"
               * which is only desired when expanded.
               *
               * To fix this we add the bottom margin, equivalent to what we would
               * have used for gap, to the AnimateHeight component's first child.
               */}
              <Flex $gap="medium" $flexDirection="column" $margin={{ bottom: 'medium' }}>
                {showDownloadLabel ? <DownloadLabel /> : null}
                {showDownloadProgress ? <DownloadProgress /> : null}
              </Flex>
            </AnimateHeight>
            <Flex $gap="medium" $flexDirection="column">
              {showReportProblemButton ? <ReportProblemButton /> : null}
              {showUpgradeButton ? <UpgradeButton /> : null}
              {showCancelButton ? <CancelButton /> : null}
            </Flex>
          </Flex>
        </StyledFooter>
      </Layout>
    </BackAction>
  );
};
