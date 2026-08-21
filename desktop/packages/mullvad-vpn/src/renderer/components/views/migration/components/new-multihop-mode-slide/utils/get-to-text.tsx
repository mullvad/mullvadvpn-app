import styled from 'styled-components';

import { messages } from '../../../../../../../shared/gettext';
import { Icon } from '../../../../../../lib/components';
import { formatHtml } from '../../../../../../lib/html-formatter';
import type { ToMultihopMode } from '../types';

const StyledIcon = styled(Icon)`
  display: inline-block;
  vertical-align: middle;
`;

export function getToText(to: ToMultihopMode) {
  switch (to) {
    case 'always':
      return {
        label: messages.gettext('Always'),
        descriptions: [
          // TRANSLATORS: Description of the new multihop mode "Always".
          messages.pgettext(
            'migration-view',
            'Multihop is enabled. Your connection is routed through an entry server before exiting through the selected location.',
          ),
        ],
      };
    case 'never':
      return {
        label: messages.gettext('Never'),
        descriptions: [
          // TRANSLATORS: Description of the new multihop mode "Never".
          messages.pgettext(
            'migration-view',
            'Multihop is disabled. Your selected location must support all active settings in order to establish a connection.',
          ),
        ],
      };
    case 'when-needed':
      return {
        label: messages.gettext('When needed'),
        descriptions: [
          // TRANSLATORS: Description of the new multihop mode "When needed".
          messages.pgettext(
            'migration-view',
            'To ensure your current settings work with your selected location, and to avoid blocking your connection, the app might automatically enable multihop via a different entry server.',
          ),
          formatHtml(
            messages.pgettext(
              'migration-view',
              'This will be indicated by the symbol: <img></img>',
            ),
            {
              img: () => (
                <StyledIcon
                  icon="magic-multihop"
                  aria-label={messages.gettext('Magic multihop')}
                  color="whiteAlpha60"
                  size="small"
                />
              ),
            },
          ),
        ],
      };
  }
}
