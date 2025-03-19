import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../shared/gettext';
import { useSuggestedUpgrade } from '../../../hooks';

export const useTitle = () => {
  const suggestedUpgrade = useSuggestedUpgrade();

  const title = sprintf(
    // TRANSLATORS: Heading which shows the version of the app which can be upgraded to.
    // TRANSLATORS: Available placeholders:
    // TRANSLATORS: %(version)s - The new version of the app.
    messages.pgettext('app-upgrade-view', 'Version %(version)s'),
    {
      version: suggestedUpgrade?.version,
    },
  );

  return title;
};
