import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { Container, Flex, TitleBig, TitleLarge } from '../../../../../lib/components';
import { formatHtml } from '../../../../../lib/html-formatter';
import { ChangelogList } from '../../../../changelog-list';
import { HeaderSubTitle } from '../../../../SettingsHeader';
import { AppUpgradeHeader } from '../app-upgrade-header';
import { NoChangelogUpdates } from './components';
import { useCacheDir, useChangelog, useShowChangelogList, useTitle } from './hooks';

export function UpgradeDetails() {
  const changelog = useChangelog();
  const showChangelogList = useShowChangelogList();
  const title = useTitle();
  const cacheDir = useCacheDir();

  const showHeaderSubtitle = cacheDir !== undefined;

  return (
    <Flex flexDirection="column" gap="large" padding={{ bottom: 'medium' }}>
      <AppUpgradeHeader>
        <TitleBig as="h2">
          {
            // TRANSLATORS: Main title for the update available view
            messages.pgettext('app-upgrade-view', 'Update available')
          }
        </TitleBig>
        {showHeaderSubtitle && (
          <HeaderSubTitle>
            {formatHtml(
              sprintf(
                // TRANSLATORS: Info about which directory the app update will be downloaded to
                messages.pgettext(
                  'app-upgrade-view',
                  'Installer will be downloaded to <b>%(cacheDir)s</b>.',
                ),
                { cacheDir },
              ),
            )}
          </HeaderSubTitle>
        )}
      </AppUpgradeHeader>
      <Flex flexDirection="column" gap="small">
        <Container horizontalMargin="medium">
          <TitleLarge as="h2">{title}</TitleLarge>
        </Container>
        <Container horizontalMargin="large" flexDirection="column">
          {showChangelogList ? <ChangelogList changelog={changelog} /> : <NoChangelogUpdates />}
        </Container>
      </Flex>
    </Flex>
  );
}
