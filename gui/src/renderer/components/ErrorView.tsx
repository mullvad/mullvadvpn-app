import React from 'react';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { measurements } from './common-styles';
import { HeaderBarSettingsButton } from './HeaderBar';
import ImageView from './ImageView';
import { Container, Header, Layout } from './Layout';

const StyledContainer = styled(Container)({
  flex: 1,
  flexDirection: 'column',
  alignItems: 'center',
  justifyContent: 'end',
});

const StyledContent = styled.div({
  display: 'flex',
  flex: 1,
  flexDirection: 'column',
  alignItems: 'center',
  justifyContent: 'end',
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
  margin: `0 ${measurements.viewMargin}`,
  color: colors.white40,
  textAlign: 'center',
});

const StyledFooterContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  justifyContent: 'end',
  minHeight: '241px',
});

interface ErrorViewProps {
  settingsUnavailable?: boolean;
  footer?: React.ReactNode | React.ReactNode[];
  children: React.ReactNode | React.ReactNode[];
}

export default function ErrorView(props: ErrorViewProps) {
  return (
    <Layout>
      <Header>{!props.settingsUnavailable && <HeaderBarSettingsButton />}</Header>
      <StyledContainer>
        <StyledContent>
          <Logo height={106} width={106} source="logo-icon" />
          <Title height={18} source="logo-text" />
          <Subtitle role="alert">{props.children}</Subtitle>
        </StyledContent>
        <StyledFooterContainer>{props.footer}</StyledFooterContainer>
      </StyledContainer>
    </Layout>
  );
}
