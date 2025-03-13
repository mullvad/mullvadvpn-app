import { messages } from '../../../../shared/gettext';
import { Container, Flex, TitleBig, TitleLarge } from '../../../lib/components';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { Changelog, Footer } from './components';
import { useSuggestedUpgrade } from './hooks';

export const DownloadUpdateView = () => {
  const { pop } = useHistory();
  const suggestedUpgrade = useSuggestedUpgrade();

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader
              title={
                // TRANSLATORS: Title in navigation bar
                messages.pgettext('download-update-view', 'Update available')
              }
            />
            <NavigationScrollbars>
              <Flex $flexDirection="column" $gap="large" $padding={{ bottom: 'medium' }}>
                <Container size="4">
                  <TitleBig as={'h2'}>
                    {
                      // TRANSLATORS: Main title for the update available view
                      messages.pgettext('download-update-view', 'Update available')
                    }
                  </TitleBig>
                </Container>
                <Flex $flexDirection="column" $gap="small">
                  <Container size="4">
                    <TitleLarge as="h2">{suggestedUpgrade?.version}</TitleLarge>
                  </Container>
                  <Container size="3" $flexDirection="column">
                    <Changelog />
                  </Container>
                </Flex>
              </Flex>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
        <Footer />
      </Layout>
    </BackAction>
  );
};
