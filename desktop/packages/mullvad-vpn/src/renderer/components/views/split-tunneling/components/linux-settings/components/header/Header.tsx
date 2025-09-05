import { strings } from '../../../../../../../../shared/constants';
import { messages } from '../../../../../../../../shared/gettext';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from '../../../../../../SettingsHeader';

export function Header() {
  return (
    <SettingsHeader>
      <HeaderTitle>{strings.splitTunneling}</HeaderTitle>
      <HeaderSubTitle>
        {messages.pgettext(
          'split-tunneling-view',
          'Click on an app to launch it. Its traffic will bypass the VPN tunnel until you close it.',
        )}
      </HeaderSubTitle>
    </SettingsHeader>
  );
}
