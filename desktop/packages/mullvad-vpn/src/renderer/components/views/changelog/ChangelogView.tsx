import { messages } from '../../../../shared/gettext';
import { Flex, TitleBig, TitleLarge } from '../../../lib/components';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { useVersionCurrent } from '../../../redux/hooks';
import { AppNavigationHeader } from '../../';
import { ChangelogList } from '../../changelog-list';
import { BackAction } from '../../KeyboardNavigation';
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
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <AppNavigationHeader
            title={
              // TRANSLATORS: Heading for the view of the changes and updates in the
              // TRANSLATORS: current version compared to the old version.
              messages.pgettext('changelog-view', 'What’s new')
            }
          />

          <NavigationScrollbars>
            <View.Content gap="large">
              <View.Container indent="medium">
                <TitleBig as="h1">
                  {
                    // TRANSLATORS: Heading for the view of the changes and updates in the
                    // TRANSLATORS: current version compared to the old version.
                    messages.pgettext('changelog-view', 'What’s new')
                  }
                </TitleBig>
              </View.Container>
              <Flex flexDirection="column" gap="small">
                <View.Container indent="medium">
                  <TitleLarge as="h2">{current}</TitleLarge>
                </View.Container>
                <View.Container indent="large" flexDirection="column">
                  {showChangelogList ? (
                    <ChangelogList changelog={changelog} />
                  ) : (
                    <NoChangelogUpdates />
                  )}
                </View.Container>
              </Flex>
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
};
