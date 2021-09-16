import styled from 'styled-components';
import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import { sourceSansPro } from './common-styles';
import { HeaderBarSettingsButton } from './HeaderBar';
import ImageView from './ImageView';
import { Container, Header, Layout } from './Layout';

const StyledContainer = styled(Container)({
  flex: 1,
  flexDirection: 'column',
  alignItems: 'center',
  justifyContent: 'center',
  marginTop: '-150px',
});

const Title = styled.h1({
  ...sourceSansPro,
  fontSize: '26px',
  lineHeight: '30px',
  color: colors.white60,
  marginBottom: '4px',
});

const Subtitle = styled.span({
  fontFamily: 'Open Sans',
  fontSize: '14px',
  lineHeight: '20px',
  marginHorizontal: '22px',
  color: colors.white40,
  textAlign: 'center',
});

const Logo = styled(ImageView)({
  marginBottom: '5px',
});

export default function Launch() {
  return (
    <Layout>
      <Header>
        <HeaderBarSettingsButton />
      </Header>
      <StyledContainer>
        <Logo height={106} width={106} source="logo-icon" />
        <Title>MULLVAD VPN</Title>
        <Subtitle role="alert">
          {messages.pgettext('launch-view', 'Connecting to Mullvad system service...')}
        </Subtitle>
      </StyledContainer>
    </Layout>
  );
}
