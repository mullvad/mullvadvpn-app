import styled from 'styled-components';

import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { Button, Flex } from '../lib/components';
import { colors } from '../lib/foundations';
import { IconBadge } from '../lib/icon-badge';
import { useSelector } from '../redux/store';
import { AppMainHeader } from './app-main-header';
import { bigText, measurements, smallText } from './common-styles';
import CustomScrollbars from './CustomScrollbars';
import { Container, Footer, Layout } from './Layout';

export const StyledCustomScrollbars = styled(CustomScrollbars)({
  flex: 1,
});

export const StyledContainer = styled(Container)({
  paddingTop: '22px',
  minHeight: '100%',
  backgroundColor: colors.darkBlue,
});

export const StyledBody = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  padding: `0 ${measurements.horizontalViewMargin}`,
});

export const StyledTitle = styled.span(bigText, {
  lineHeight: '38px',
  marginBottom: '8px',
  color: colors.white,
});

export const StyledMessage = styled.span(smallText, {
  marginBottom: measurements.rowVerticalMargin,
  color: colors.white,
});

export function DeviceRevokedView() {
  const { leaveRevokedDevice } = useAppContext();
  const tunnelState = useSelector((state) => state.connection.status);

  return (
    <Layout>
      <AppMainHeader variant="basedOnConnectionStatus" size="basedOnLoginStatus">
        <AppMainHeader.AccountButton />
        <AppMainHeader.SettingsButton />
      </AppMainHeader>
      <StyledCustomScrollbars fillContainer>
        <StyledContainer>
          <StyledBody>
            <Flex justifyContent="center" margin={{ bottom: 'medium' }}>
              <IconBadge state="negative" />
            </Flex>
            <StyledTitle data-testid="title">
              {messages.pgettext('device-management', 'Device is inactive')}
            </StyledTitle>
            <StyledMessage>
              {messages.pgettext(
                'device-management',
                'You have removed this device. To connect again, you will need to log back in.',
              )}
            </StyledMessage>
            <StyledMessage>
              {tunnelState.state !== 'disconnected' &&
                messages.pgettext(
                  'device-management',
                  'Going to login will unblock the Internet on this device.',
                )}
            </StyledMessage>
          </StyledBody>

          <Footer>
            <Button
              variant={tunnelState.state === 'disconnected' ? 'primary' : 'destructive'}
              onClick={leaveRevokedDevice}>
              <Button.Text>
                {
                  // TRANSLATORS: Button label for navigating to login.
                  messages.pgettext('device-management', 'Go to login')
                }
              </Button.Text>
            </Button>
          </Footer>
        </StyledContainer>
      </StyledCustomScrollbars>
    </Layout>
  );
}
