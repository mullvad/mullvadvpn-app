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
  ReportProblemButton,
  RetryUpgradeButton,
  UpgradeButton,
  UpgradeDetails,
  UpgradeLabel,
} from './components';
import {
  usePresent,
  useShowCancelButton,
  useShowDownloadProgress,
  useShowInstallButton,
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
  const present = usePresent();
  const showCancelButton = useShowCancelButton();
  const showDownloadProgress = useShowDownloadProgress();
  const showInstallButton = useShowInstallButton();
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
            <Animate
              animations={[{ type: 'fade' }, { type: 'wipe', direction: 'vertical' }]}
              present={present}>
              <Flex $gap="medium" $flexDirection="column" $margin={{ bottom: 'medium' }}>
                {showUpgradeLabel && <UpgradeLabel />}
                {showDownloadProgress && <DownloadProgress />}
              </Flex>
            </Animate>
            <Flex $gap="medium" $flexDirection="column">
              {showReportProblemButton && <ReportProblemButton />}
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
