import styled from 'styled-components';

import { Animate } from '../../../lib/components/animate';
import { useHistory } from '../../../lib/history';
import { BackAction } from '../../KeyboardNavigation';
import { Layout } from '../../Layout';
import { Footer, UpgradeDetails } from './components';

const StyledFooter = styled.div`
  // TODO: Use color from Colors
  background-color: rgba(21, 39, 58, 1);
  position: sticky;
  bottom: 0;
  width: 100%;
`;

export const AppUpgradeView = () => {
  const { pop } = useHistory();

  return (
    <BackAction action={pop}>
      <Layout>
        <UpgradeDetails />
        <StyledFooter>
          <Animate animations={[{ type: 'wipe', direction: 'vertical' }]} present>
            <Footer />
          </Animate>
        </StyledFooter>
      </Layout>
    </BackAction>
  );
};
