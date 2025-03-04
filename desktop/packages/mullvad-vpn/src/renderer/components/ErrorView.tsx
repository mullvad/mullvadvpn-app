import React from 'react';
import styled from 'styled-components';

import { Flex, Logo } from '../lib/components';
import { Colors, Spacings } from '../lib/foundations';
import { AppMainHeader } from './app-main-header';
import { measurements } from './common-styles';
import { Container, Layout } from './Layout';

const StyledContainer = styled(Container)({
  flex: 1,
  flexDirection: 'column',
  alignItems: 'center',
  justifyContent: 'end',
});

const Subtitle = styled.span({
  fontFamily: 'Open Sans',
  fontSize: '14px',
  lineHeight: '20px',
  margin: `0 ${measurements.horizontalViewMargin}`,
  color: Colors.white40,
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
      <AppMainHeader logoVariant="none">
        {!props.settingsUnavailable && <AppMainHeader.SettingsButton />}
      </AppMainHeader>
      <StyledContainer>
        <Flex $flexDirection="column" $gap={Spacings.small}>
          <Flex
            $flexDirection="column"
            $alignItems="center"
            $justifyContent="end"
            $gap={Spacings.medium}>
            <Logo variant="icon" size="2" />
            <Logo variant="text" size="2" />
          </Flex>
          <Subtitle role="alert">{props.children}</Subtitle>
          <StyledFooterContainer>{props.footer}</StyledFooterContainer>
        </Flex>
      </StyledContainer>
    </Layout>
  );
}
