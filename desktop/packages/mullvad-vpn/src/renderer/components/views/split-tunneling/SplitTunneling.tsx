import { strings } from '../../../../shared/constants';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { LinuxSettings, Settings } from './components';
import { SplitTunnelingContextProvider, useSplitTunnelingContext } from './SplitTunnelingContext';
import { StyledNavigationScrollbars, StyledPageCover } from './SplitTunnelingStyles';

function SplitTunnelingInner() {
  const { pop } = useHistory();
  const { browsing, scrollbarsRef } = useSplitTunnelingContext();
  const showLinuxSettings = window.env.platform === 'linux';

  return (
    <>
      <StyledPageCover $show={browsing} />
      <BackAction action={pop}>
        <Layout>
          <SettingsContainer>
            <NavigationContainer>
              <AppNavigationHeader title={strings.splitTunneling} />
              <StyledNavigationScrollbars ref={scrollbarsRef}>
                {showLinuxSettings ? <LinuxSettings /> : <Settings />}
              </StyledNavigationScrollbars>
            </NavigationContainer>
          </SettingsContainer>
        </Layout>
      </BackAction>
    </>
  );
}

export function SplitTunneling() {
  return (
    <SplitTunnelingContextProvider>
      <SplitTunnelingInner />
    </SplitTunnelingContextProvider>
  );
}
