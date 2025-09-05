import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../../../shared/constants';
import { messages } from '../../../../../../../../shared/gettext';
import { Link } from '../../../../../../../lib/components';
import { useLinuxSettingsContext } from '../../LinuxSettingsContext';
import { useShowUnsupportedDialog } from './hooks';

export function HeaderDescription() {
  const { splitTunnelingSupported } = useLinuxSettingsContext();
  const showUnsupportedDialog = useShowUnsupportedDialog();

  if (splitTunnelingSupported === false) {
    return (
      <>
        {sprintf(
          // TRANSLATORS: Information about split tunneling not being supported on the system.
          // TRANSLATORS: Available placeholders:
          // TRANSLATORS: %(splitTunneling)s - will be replaced with Split tunneling
          messages.pgettext(
            'split-tunneling-view',
            '%(splitTunneling)s is not supported by your system.',
          ),
          {
            splitTunneling: strings.splitTunneling,
          },
        )}
        &nbsp;
        <Link variant="labelTiny" onClick={showUnsupportedDialog}>
          <Link.Text>
            {
              // TRANSLATORS: Link for learning more
              messages.pgettext('split-tunneling-view', 'Click here to learn more')
            }
          </Link.Text>
        </Link>
      </>
    );
  }

  return messages.pgettext(
    'split-tunneling-view',
    'Click on an app to launch it. Its traffic will bypass the VPN tunnel until you close it.',
  );
}
