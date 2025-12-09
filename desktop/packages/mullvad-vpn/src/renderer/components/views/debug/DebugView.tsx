import { useCallback } from 'react';

import { Button } from '../../../lib/components';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { useBoolean } from '../../../lib/utility-hooks';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../KeyboardNavigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';

export function DebugView() {
  const { pop } = useHistory();

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <AppNavigationHeader title="Developer tools" />

          <NavigationScrollbars>
            <View.Content>
              <SettingsHeader>
                <HeaderTitle>Developer tools</HeaderTitle>
              </SettingsHeader>

              <View.Container marginInline="large" flexDirection="column" gap="medium">
                <ThrowErrorButton />
                <UnhandledRejectionButton />
                <ErrorDuringRender />
              </View.Container>
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
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
