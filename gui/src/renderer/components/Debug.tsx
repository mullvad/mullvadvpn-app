import { useCallback } from 'react';
import styled from 'styled-components';

import { useHistory } from '../lib/history';
import { useBoolean } from '../lib/utilityHooks';
import * as AppButton from './AppButton';
import { measurements } from './common-styles';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';
import {
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  marginBottom: '2px',
});

const StyledButtonGroup = styled.div({
  margin: measurements.viewMargin,
});

export default function Debug() {
  const { pop } = useHistory();

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <NavigationBar>
              <NavigationItems>
                <TitleBarItem>Developer tools</TitleBarItem>
              </NavigationItems>
            </NavigationBar>

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>Developer tools</HeaderTitle>
              </SettingsHeader>

              <StyledContent>
                <StyledButtonGroup>
                  <AppButton.ButtonGroup>
                    <ThrowErrorButton />
                    <UnhandledRejectionButton />
                    <ErrorDuringRender />
                  </AppButton.ButtonGroup>
                </StyledButtonGroup>
              </StyledContent>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

function ThrowErrorButton() {
  const handleClick = useCallback(() => {
    throw new Error('This is a test error');
  }, []);

  return <AppButton.RedButton onClick={handleClick}>Throw error</AppButton.RedButton>;
}

function UnhandledRejectionButton() {
  const handleClick = useCallback(() => {
    return new Promise((_resolve, reject) => setTimeout(reject, 100));
  }, []);

  return <AppButton.RedButton onClick={handleClick}>Unhandled rejection</AppButton.RedButton>;
}

function ErrorDuringRender() {
  const [error, setError] = useBoolean(false);

  if (error) {
    throw new Error('This is a test error during render');
  }

  return <AppButton.RedButton onClick={setError}>Error next render</AppButton.RedButton>;
}
