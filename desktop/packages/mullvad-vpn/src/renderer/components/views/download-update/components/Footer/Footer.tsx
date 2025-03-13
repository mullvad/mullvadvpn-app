import styled from 'styled-components';

import { Flex } from '../../../../../lib/components';
import { AnimateHeight } from '../../../../AnimateHeight';
import { CancelButton, DownloadProgress, Label, UpgradeButton } from './components';

const StyledFlex = styled(Flex)`
  background-color: rgba(21, 39, 58, 1);
  position: sticky;
  bottom: 0;
  width: 100%;
`;

export const Footer = () => {
  const showUpgradeButton = false;
  const showCancelButton = false;

  return (
    <StyledFlex $padding="medium" $flexDirection="column">
      <AnimateHeight>
        <Label />
        <DownloadProgress />
      </AnimateHeight>
      {showUpgradeButton ? <UpgradeButton /> : null}
      {showCancelButton ? <CancelButton /> : null}
    </StyledFlex>
  );
};
