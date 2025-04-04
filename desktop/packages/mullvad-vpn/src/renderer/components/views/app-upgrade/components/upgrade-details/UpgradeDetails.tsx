import { messages } from '../../../../../../shared/gettext';
import { Container, Flex, TitleBig, TitleLarge } from '../../../../../lib/components';
import { AppNavigationHeader } from '../../../../app-navigation-header';
import { ChangelogList } from '../../../../changelog-list';
import { SettingsContainer } from '../../../../Layout';
import { NavigationContainer } from '../../../../NavigationContainer';
import { NavigationScrollbars } from '../../../../NavigationScrollbars';
import { NoChangelogUpdates } from './components';
import { useChangelog, useShowChangelogList, useShowNoChangelogUpdates, useTitle } from './hooks';

export function UpgradeDetails() {
  const changelog = useChangelog();
  const showChangelogList = useShowChangelogList();
  const showNoChangelogUpdates = useShowNoChangelogUpdates();
  const title = useTitle();

  return (
    <SettingsContainer>
      <NavigationContainer>
        <AppNavigationHeader
          title={
            // TRANSLATORS: Title in navigation bar
            messages.pgettext('app-upgrade-view', 'Update available')
          }
        />
        <NavigationScrollbars>
          <Flex $flexDirection="column" $gap="large" $padding={{ bottom: 'medium' }}>
            <Container size="4">
              <TitleBig as="h2">
                {
                  // TRANSLATORS: Main title for the update available view
                  messages.pgettext('app-upgrade-view', 'Update available')
                }
              </TitleBig>
            </Container>
            <Flex $flexDirection="column" $gap="small">
              <Container size="4">
                <TitleLarge as="h2">{title}</TitleLarge>
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
  );
}
