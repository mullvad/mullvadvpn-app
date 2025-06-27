import { Flex, Logo } from '../../../lib/components';
import { View } from '../../../lib/components/view';
import { AppMainHeader } from '../../app-main-header';
import { Footer, StatusText } from './components';

export function LaunchView() {
  return (
    <View>
      <AppMainHeader logoVariant="none">
        <AppMainHeader.SettingsButton />
      </AppMainHeader>
      <View.Container size="4" $flex={1}>
        <Flex
          $flexDirection="column"
          $flex={1}
          $margin={{ vertical: 'large' }}
          $alignItems="center"
          $gap="medium">
          <Flex $flexDirection="column" $gap="medium">
            <Logo variant="icon" size="2" />
            <Logo variant="text" size="2" />
          </Flex>
          <StatusText />
        </Flex>
        <Flex $margin={{ vertical: 'large' }}>
          <Footer />
        </Flex>
      </View.Container>
    </View>
  );
}
