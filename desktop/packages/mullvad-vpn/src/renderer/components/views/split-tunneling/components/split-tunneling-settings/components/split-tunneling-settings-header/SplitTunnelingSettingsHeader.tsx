import { strings } from '../../../../../../../../shared/constants';
import { messages } from '../../../../../../../../shared/gettext';
import { Flex } from '../../../../../../../lib/components';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from '../../../../../../SettingsHeader';
import { MacOsSplitTunnelingAvailability, SplitTunnelingStateSwitch } from './components';
import { useShowMacOsSplitTunnelingAvailability } from './hooks';
import { useShowHeaderSubtitle } from './hooks';

export function SplitTunnelingSettingsHeader() {
  const showHeaderSubtitle = useShowHeaderSubtitle();
  const showMacOsSplitTunnelingAvailability = useShowMacOsSplitTunnelingAvailability();

  return (
    <SettingsHeader>
      <Flex justifyContent="space-between" alignItems="center">
        <HeaderTitle>{strings.splitTunneling}</HeaderTitle>
        <SplitTunnelingStateSwitch />
      </Flex>
      {showMacOsSplitTunnelingAvailability && <MacOsSplitTunnelingAvailability />}
      {showHeaderSubtitle && (
        <HeaderSubTitle>
          {messages.pgettext(
            'split-tunneling-view',
            'Choose the apps you want to exclude from the VPN tunnel.',
          )}
        </HeaderSubTitle>
      )}
    </SettingsHeader>
  );
}
