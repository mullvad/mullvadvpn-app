import styled from 'styled-components';
import { colors } from '../../config.json';
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

const Logo = styled(ImageView)({
  marginBottom: '12px',
});

const Title = styled(ImageView)({
  opacity: 0.6,
  marginBottom: '9px',
});

const Subtitle = styled.span({
  fontFamily: 'Open Sans',
  fontSize: '14px',
  lineHeight: '20px',
  marginHorizontal: '22px',
  color: colors.white40,
  textAlign: 'center',
});

interface ErrorViewProps {
  settingsUnavailable?: boolean;
  children: React.ReactNode | React.ReactNode[];
}

export default function ErrorView(props: ErrorViewProps) {
  return (
    <Layout>
      <Header>{!props.settingsUnavailable && <HeaderBarSettingsButton />}</Header>
      <StyledContainer>
        <Logo height={106} width={106} source="logo-icon" />
        <Title height={18} source="logo-text" />
        <Subtitle role="alert">{props.children}</Subtitle>
      </StyledContainer>
    </Layout>
  );
}
