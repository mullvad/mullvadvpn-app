import styled from 'styled-components';

import { Flex } from '../../../lib/components';
import { Animate } from '../../../lib/components/animate';
import { useHistory } from '../../../lib/history';
import { BackAction } from '../../KeyboardNavigation';
import { Layout } from '../../Layout';
import {
  CancelButton,
  DownloadProgress,
  InstallButton,
  ManualDownloadButton,
  ReportProblemButton,
  RetryUpgradeButton,
  UpgradeButton,
  UpgradeDetails,
  UpgradeLabel,
} from './components';
import {
  useShowCancelButton,
  useShowDownloadProgress,
  useShowInstallButton,
  useShowManualDownloadButton,
  useShowReportProblemButton,
  useShowRetryUpgradeButton,
  useShowUpgradeButton,
  useShowUpgradeLabel,
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
  const showDownloadProgress = useShowDownloadProgress();
  const showInstallButton = useShowInstallButton();
  const showManualDownloadButton = useShowManualDownloadButton();
  const showReportProblemButton = useShowReportProblemButton();
  const showRetryUpgradeButton = useShowRetryUpgradeButton();
  const showUpgradeButton = useShowUpgradeButton();
  const showUpgradeLabel = useShowUpgradeLabel();

  return (
    <BackAction action={pop}>
      <Layout>
        <UpgradeDetails />
        <StyledFooter>
          <Flex $padding="large" $flexDirection="column">
            <Flex $flexDirection="column">
              <Animate
                animations={[{ type: 'wipe', direction: 'vertical' }]}
                present={showUpgradeLabel}>
                <Flex $margin={{ bottom: 'medium' }}>
                  <UpgradeLabel />
                </Flex>
              </Animate>
              <Animate
                animations={[{ type: 'wipe', direction: 'vertical' }, { type: 'fade' }]}
                present={showDownloadProgress}>
                <Flex $margin={{ bottom: 'medium' }}>
                  <DownloadProgress />
                </Flex>
              </Animate>
            </Flex>
            <Flex $gap="medium" $flexDirection="column">
              {showReportProblemButton && <ReportProblemButton />}
              {showManualDownloadButton && <ManualDownloadButton />}
              {showRetryUpgradeButton && <RetryUpgradeButton />}
              {showUpgradeButton && <UpgradeButton />}
              {showInstallButton && <InstallButton />}
              {showCancelButton && <CancelButton />}
            </Flex>
          </Flex>
        </StyledFooter>
      </Layout>
    </BackAction>
  );
};
