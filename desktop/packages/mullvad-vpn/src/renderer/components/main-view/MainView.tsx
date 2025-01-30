import styled from 'styled-components';

import { Spinner } from '../../lib/components';
import { useSelector } from '../../redux/store';
import { AppMainHeader } from '../app-main-header';
import { Container, Layout } from '../Layout';
import Map from '../Map';
import NotificationArea from '../NotificationArea';
import ConnectionPanel from './ConnectionPanel';

const StyledContainer = styled(Container)({
  position: 'relative',
});

const Content = styled.div({
  display: 'flex',
  flex: 1,
  flexDirection: 'column',
  position: 'relative', // need this for z-index to work to cover the map
  zIndex: 1,
  maxHeight: '100%',
});

const StatusIcon = styled(Spinner)({
  position: 'absolute',
  alignSelf: 'center',
  marginTop: 94,
});

const StyledNotificationArea = styled(NotificationArea)({
  position: 'absolute',
  left: 0,
  top: 0,
  right: 0,
});

const StyledMain = styled.main({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  maxHeight: '100%',
});

export default function MainView() {
  const connection = useSelector((state) => state.connection);

  const showSpinner =
    connection.status.state === 'connecting' || connection.status.state === 'disconnecting';

  return (
    <Layout>
      <AppMainHeader size="basedOnLoginStatus" variant="basedOnConnectionStatus">
        <AppMainHeader.AccountButton />
        <AppMainHeader.SettingsButton />
      </AppMainHeader>
      <StyledContainer>
        <Map />
        <Content>
          <StyledNotificationArea />

          <StyledMain>
            {showSpinner ? <StatusIcon size="big" /> : null}

            <ConnectionPanel />
          </StyledMain>
        </Content>
      </StyledContainer>
    </Layout>
  );
}
