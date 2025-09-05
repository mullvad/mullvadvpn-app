import { messages } from '../../../../../../../../../../shared/gettext';
import { Link } from '../../../../../../../../../lib/components';
import { useSettingsSplitTunnelingSupported } from '../../../../../../../../../redux/hooks';
import { useShowUnsupportedDialog } from './hooks';

export function Subtitle() {
  const { splitTunnelingSupported } = useSettingsSplitTunnelingSupported();
  const showUnsupportedDialog = useShowUnsupportedDialog();

  if (!splitTunnelingSupported) {
    <>
      {messages.pgettext(
        'split-tunneling-view',
        'Split tunneling is not supported by your system.',
      )}
      &nbsp;
      <Link onClick={showUnsupportedDialog}>
        {messages.pgettext('split-tunneling-view', 'Click here to learn more')}
      </Link>
    </>;
  }

  return messages.pgettext(
    'split-tunneling-view',
    'Click on an app to launch it. Its traffic will bypass the VPN tunnel until you close it.',
  );
}
