import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { TitleBig } from '../../../../../lib/components';
import { EmptyState } from '../../../../../lib/components/empty-state';
import { useVersionCurrent } from '../../../../../redux/hooks';
import { AppUpgradeHeader } from '../app-upgrade-header';

export function NoUpgradeAvailable() {
  const { current } = useVersionCurrent();

  return (
    <AppUpgradeHeader>
      <TitleBig as="h2">
        {
          // TRANSLATORS: Main title for the update available view
          messages.pgettext('app-upgrade-view', 'Update available')
        }
      </TitleBig>
      <EmptyState
        variant="success"
        $alignSelf="stretch"
        $margin={{
          top: 'medium',
        }}>
        <EmptyState.StatusIcon />
        <EmptyState.TextContainer>
          <EmptyState.Title>
            {
              // TRANSLATORS: The user has the latest version of the app and does not need to be update
              messages.pgettext('app-upgrade-view', 'You are using the latest version')
            }
          </EmptyState.Title>
          <EmptyState.Subtitle>
            {
              // TRANSLATORS: The the latest version of the app
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(version)s - The version number of the latest app version
              sprintf(messages.pgettext('app-upgrade-view', 'Latest version: %(version)'), {
                version: current,
              })
            }
          </EmptyState.Subtitle>
        </EmptyState.TextContainer>
      </EmptyState>
    </AppUpgradeHeader>
  );
}
