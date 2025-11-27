import { messages } from '../../../../../../../../../../shared/gettext';
import { useAppContext } from '../../../../../../../../../context';
import { Button, Flex, FootnoteMini } from '../../../../../../../../../lib/components';
import { HeaderSubTitle } from '../../../../../../../../SettingsHeader';
import { useRestartDaemon } from './hooks';

export function MacOsSplitTunnelingAvailability() {
  const { showFullDiskAccessSettings } = useAppContext();
  const restartDaemon = useRestartDaemon();

  return (
    <Flex flexDirection="column" gap="large">
      <HeaderSubTitle>
        {messages.pgettext(
          'split-tunneling-view',
          'To use split tunneling please enable “Full disk access” for “Mullvad VPN” in the macOS system settings.',
        )}
      </HeaderSubTitle>
      <Flex flexDirection="column" gap="small">
        <Flex flexDirection="column" gap="big">
          <Button onClick={showFullDiskAccessSettings}>
            <Button.Text>
              {messages.pgettext('split-tunneling-view', 'Open System Settings')}
            </Button.Text>
          </Button>
          <FootnoteMini color="whiteAlpha60">
            {messages.pgettext(
              'split-tunneling-view',
              'Enabled "Full disk access" and still having issues?',
            )}
          </FootnoteMini>
        </Flex>
        <Button onClick={restartDaemon}>
          <Button.Text>
            {messages.pgettext('split-tunneling-view', 'Restart Mullvad Service')}
          </Button.Text>
        </Button>
      </Flex>
    </Flex>
  );
}
