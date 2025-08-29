import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { Container, Flex, TitleBig, TitleLarge } from '../../../../../lib/components';
import { formatHtml } from '../../../../../lib/html-formatter';
import { AppNavigationHeader } from '../../../../app-navigation-header';
import { ChangelogList } from '../../../../changelog-list';
import { SettingsContainer } from '../../../../Layout';
import { NavigationContainer } from '../../../../NavigationContainer';
import { NavigationScrollbars } from '../../../../NavigationScrollbars';
import { HeaderSubTitle } from '../../../../SettingsHeader';
import { NoChangelogUpdates } from './components';
import { useCacheDir, useChangelog, useShowChangelogList, useTitle } from './hooks';

export function UpgradeDetails() {
  const changelog = useChangelog();
  const showChangelogList = useShowChangelogList();
  const title = useTitle();
  const cacheDir = useCacheDir();

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
            <Container size="4" $flexDirection="column" $gap="small">
              <TitleBig as="h2">
                {
                  // TRANSLATORS: Main title for the update available view
                  messages.pgettext('app-upgrade-view', 'Update available')
                }
              </TitleBig>
              {cacheDir !== undefined && (
                <HeaderSubTitle>
                  {formatHtml(
                    sprintf(
                      messages.pgettext(
                        'app-upgrade-view',
                        'Installer will be downloaded to <b>%(cacheDir)s</b>.',
                      ),
                      { cacheDir },
                    ),
                  )}
                </HeaderSubTitle>
              )}
            </Container>
            <Flex $flexDirection="column" $gap="small">
              <Container size="4">
                <TitleLarge as="h2">{title}</TitleLarge>
              </Container>
              <Container size="3" $flexDirection="column">
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
  );
}
