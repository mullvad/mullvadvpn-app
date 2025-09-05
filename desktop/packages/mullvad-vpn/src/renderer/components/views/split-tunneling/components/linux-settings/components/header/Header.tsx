import { strings } from '../../../../../../../../shared/constants';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from '../../../../../../SettingsHeader';
import { Subtitle, UnsupportedDialog } from './components';
import { HeaderContextProvider } from './HeaderContext';

function HeaderInner() {
  return (
    <>
      <SettingsHeader>
        <HeaderTitle>{strings.splitTunneling}</HeaderTitle>
        <HeaderSubTitle>
          <Subtitle />
        </HeaderSubTitle>
      </SettingsHeader>
      <UnsupportedDialog />
    </>
  );
}

export function Header() {
  return (
    <HeaderContextProvider>
      <HeaderInner />
    </HeaderContextProvider>
  );
}
