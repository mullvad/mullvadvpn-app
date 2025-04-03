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
import { useChangelog, useShowChangelogList, useShowNoChangelogUpdates } from './hooks';

export const ChangelogView = () => {
  const { pop } = useHistory();
  const { current } = useVersionCurrent();
  const changelog = useChangelog();
  const showChangelogList = useShowChangelogList();
  const showNoChangelogUpdates = useShowNoChangelogUpdates();

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
              <Flex $flexDirection="column" $gap="large">
                <Container size="4">
                  <TitleBig as="h1">
                    {
                      // TRANSLATORS: Heading for the view of the changes and updates in the
                      // TRANSLATORS: current version compared to the old version.
                      messages.pgettext('changelog-view', 'What’s new')
                    }
                  </TitleBig>
                </Container>
                <Flex $flexDirection="column" $gap="small">
                  <Container size="4">
                    <TitleLarge as="h2">{current}</TitleLarge>
                  </Container>
                  <Container size="3" $flexDirection="column">
                    {showChangelogList && <ChangelogList changelog={changelog} />}
                    {showNoChangelogUpdates && <NoChangelogUpdates />}
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
