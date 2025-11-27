import { useCallback } from 'react';

import { messages } from '../../../../../../../../shared/gettext';
import { useAppContext } from '../../../../../../../context';
import { Button } from '../../../../../../../lib/components';
import { FlexColumn } from '../../../../../../../lib/components/flex-column';
import { FooterText } from '../footer-text';

export function MacOsPermissionFooter() {
  const { showLaunchDaemonSettings } = useAppContext();

  const openSettings = useCallback(async () => {
    await showLaunchDaemonSettings();
  }, [showLaunchDaemonSettings]);

  return (
    <>
      <FlexColumn gap="medium">
        <FooterText>
          {
            // TRANSLATORS: Message in launch view when the background process permissions have been revoked.
            messages.pgettext(
              'launch-view',
              'Permission for the Mullvad VPN service has been revoked. Please go to System Settings and allow Mullvad VPN under the “Allow in the Background” setting.',
            )
          }
        </FooterText>
        <Button onClick={openSettings}>
          <Button.Text>
            {
              // TRANSLATORS: Button label for system settings.
              messages.gettext('Go to System Settings')
            }
          </Button.Text>
        </Button>
      </FlexColumn>
    </>
  );
}
