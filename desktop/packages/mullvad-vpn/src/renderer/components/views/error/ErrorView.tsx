import React from 'react';
import styled from 'styled-components';

import { Logo, Text } from '../../../lib/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { AppMainHeader } from '../../app-main-header';

const StyledFooterContainer = styled(FlexColumn)`
  min-height: 241px;
`;

interface ErrorViewProps {
  settingsUnavailable?: boolean;
  footer?: React.ReactNode | React.ReactNode[];
  children: React.ReactNode | React.ReactNode[];
}

export function ErrorView({ settingsUnavailable, footer, children }: ErrorViewProps) {
  return (
    <View>
      <AppMainHeader logoVariant="none">
        {!settingsUnavailable && <AppMainHeader.SettingsButton />}
      </AppMainHeader>
      <View.Container marginInline="medium" flexGrow={1} alignItems="center" justifyContent="end">
        <FlexColumn gap="medium">
          <FlexColumn alignItems="center" justifyContent="end" gap="medium">
            <Logo variant="icon" size="2" />
            <Logo variant="text" size="2" />
          </FlexColumn>
          <Text role="alert" variant="bodySmall" textAlign="center" color="whiteAlpha60">
            {children}
          </Text>
          <StyledFooterContainer justifyContent="end">{footer}</StyledFooterContainer>
        </FlexColumn>
      </View.Container>
    </View>
  );
}
