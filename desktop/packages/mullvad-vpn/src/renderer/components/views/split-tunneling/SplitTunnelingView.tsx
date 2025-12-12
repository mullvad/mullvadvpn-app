import styled from 'styled-components';

import { strings } from '../../../../shared/constants';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../keyboard-navigation';
import { Layout, SettingsContainer } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { LinuxSettings, Settings } from './components';
import { SplitTunnelingContextProvider, useSplitTunnelingContext } from './SplitTunnelingContext';

const StyledPageCover = styled.div<{ $show: boolean }>((props) => ({
  position: 'absolute',
  zIndex: 2,
  top: 0,
  left: 0,
  right: 0,
  bottom: 0,
  opacity: 0.5,
  display: props.$show ? 'block' : 'none',
}));

const StyledNavigationScrollbars = styled(NavigationScrollbars)({
  flex: 1,
});

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

export function SplitTunnelingView() {
  return (
    <SplitTunnelingContextProvider>
      <SplitTunnelingInner />
    </SplitTunnelingContextProvider>
  );
}
