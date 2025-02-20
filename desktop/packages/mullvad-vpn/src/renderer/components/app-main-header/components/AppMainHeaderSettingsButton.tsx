import { useCallback } from 'react';
import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { IconButton, IconButtonProps, MainHeader } from '../../../lib/components';
import { Notification } from '../../../lib/components/notification';
import { transitions, useHistory } from '../../../lib/history';
import { RoutePath } from '../../../lib/routes';
import { useSelector } from '../../../redux/store';
export type MainHeaderSettingsButtonProps = Omit<IconButtonProps, 'icon'>;

const StyledNotification = styled(Notification)`
  position: absolute;
  top: 0;
  right: 0;
`;

const StyledDiv = styled.div`
  position: relative;
`;

export function AppMainHeaderSettingsButton(props: MainHeaderSettingsButtonProps) {
  const history = useHistory();
  const suggestedUpgrade = useSelector((state) => state.version.suggestedUpgrade);

  const openSettings = useCallback(() => {
    if (!props.disabled) {
      history.push(RoutePath.settings, { transition: transitions.show });
    }
  }, [history, props.disabled]);

  return (
    <MainHeader.IconButton onClick={openSettings} aria-label={messages.gettext('Settings')}>
      {suggestedUpgrade ? (
        <StyledDiv>
          <IconButton.Icon icon="settings-partial" />
          <StyledNotification variant="warning" size="tiny" />
        </StyledDiv>
      ) : (
        <IconButton.Icon icon="settings-filled" />
      )}
    </MainHeader.IconButton>
  );
}
