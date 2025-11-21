import styled from 'styled-components';

import { strings } from '../../../../shared/constants';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../KeyboardNavigation';
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
      <View backgroundColor="darkBlue">
        <BackAction action={pop}>
          <NavigationContainer>
            <AppNavigationHeader title={strings.splitTunneling} />
            <StyledNavigationScrollbars ref={scrollbarsRef}>
              <View.Content>{showLinuxSettings ? <LinuxSettings /> : <Settings />}</View.Content>
            </StyledNavigationScrollbars>
          </NavigationContainer>
        </BackAction>
      </View>
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
