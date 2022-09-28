import styled from 'styled-components';

import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { useSelector } from '../redux/store';
import * as AppButton from './AppButton';
import { bigText, measurements, smallText } from './common-styles';
import CustomScrollbars from './CustomScrollbars';
import { calculateHeaderBarStyle, DefaultHeaderBar } from './HeaderBar';
import ImageView from './ImageView';
import { Container, Footer } from './Layout';
import { Layout } from './Layout';

export const StyledHeader = styled(DefaultHeaderBar)({
  flex: 0,
});

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
  padding: `0 ${measurements.viewMargin}`,
});

export const StyledStatusIcon = styled.div({
  alignSelf: 'center',
  width: '60px',
  height: '60px',
  marginBottom: '18px',
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

  const Button = tunnelState.state === 'disconnected' ? AppButton.GreenButton : AppButton.RedButton;

  return (
    <Layout>
      <StyledHeader barStyle={calculateHeaderBarStyle(tunnelState)} />
      <StyledCustomScrollbars fillContainer>
        <StyledContainer>
          <StyledBody>
            <StyledStatusIcon>
              <ImageView source="icon-fail" height={60} width={60} />
            </StyledStatusIcon>
            <StyledTitle>
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
            <Button onClick={leaveRevokedDevice}>
              {messages.pgettext('device-management', 'Go to login')}
            </Button>
          </Footer>
        </StyledContainer>
      </StyledCustomScrollbars>
    </Layout>
  );
}
