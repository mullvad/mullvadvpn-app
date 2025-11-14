import { messages } from '../../../../shared/gettext';
import { Container, Flex, TitleBig, TitleLarge } from '../../../lib/components';
import { useHistory } from '../../../lib/history';
import { useVersionCurrent } from '../../../redux/hooks';
import { AppNavigationHeader } from '../../';
import { ChangelogList } from '../../changelog-list';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { NoChangelogUpdates } from './components';
import { useChangelog, useShowChangelogList } from './hooks';

export const ChangelogView = () => {
  const { pop } = useHistory();
  const { current } = useVersionCurrent();
  const changelog = useChangelog();
  const showChangelogList = useShowChangelogList();

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader
              title={
                // TRANSLATORS: Heading for the view of the changes and updates in the
                // TRANSLATORS: current version compared to the old version.
                messages.pgettext('changelog-view', 'What’s new')
              }
            />

            <NavigationScrollbars>
              <Flex flexDirection="column" gap="large">
                <Container indent="medium">
                  <TitleBig as="h1">
                    {
                      // TRANSLATORS: Heading for the view of the changes and updates in the
                      // TRANSLATORS: current version compared to the old version.
                      messages.pgettext('changelog-view', 'What’s new')
                    }
                  </TitleBig>
                </Container>
                <Flex flexDirection="column" gap="small">
                  <Container indent="medium">
                    <TitleLarge as="h2">{current}</TitleLarge>
                  </Container>
                  <Container indent="large" flexDirection="column">
                    {showChangelogList ? (
                      <ChangelogList changelog={changelog} />
                    ) : (
                      <NoChangelogUpdates />
                    )}
                  </Container>
                </Flex>
              </Flex>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
};
