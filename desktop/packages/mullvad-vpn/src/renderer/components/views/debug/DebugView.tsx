import { useCallback } from 'react';
import styled from 'styled-components';

import { Button } from '../../../lib/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { spacings } from '../../../lib/foundations';
import { useHistory } from '../../../lib/history';
import { useBoolean } from '../../../lib/utility-hooks';
import { AppNavigationHeader } from '../..';
import { measurements } from '../../common-styles';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';

const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  marginBottom: '2px',
});

const StyledButtonGroup = styled.div({
  margin: `${spacings.large} ${measurements.horizontalViewMargin}`,
});

export function DebugView() {
  const { pop } = useHistory();

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader title="Developer tools" />

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>Developer tools</HeaderTitle>
              </SettingsHeader>

              <StyledContent>
                <StyledButtonGroup>
                  <FlexColumn gap="medium">
                    <ThrowErrorButton />
                    <UnhandledRejectionButton />
                    <ErrorDuringRender />
                  </FlexColumn>
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

  return (
    <Button variant="destructive" onClick={handleClick}>
      <Button.Text>Throw error</Button.Text>
    </Button>
  );
}

function UnhandledRejectionButton() {
  const handleClick = useCallback(() => {
    return new Promise((_resolve, reject) => setTimeout(reject, 100));
  }, []);

  return (
    <Button variant="destructive" onClick={handleClick}>
      <Button.Text>Unhandled rejection</Button.Text>
    </Button>
  );
}

function ErrorDuringRender() {
  const [error, setError] = useBoolean(false);

  if (error) {
    throw new Error('This is a test error during render');
  }

  return (
    <Button variant="destructive" onClick={setError}>
      <Button.Text>Error next render</Button.Text>
    </Button>
  );
}
