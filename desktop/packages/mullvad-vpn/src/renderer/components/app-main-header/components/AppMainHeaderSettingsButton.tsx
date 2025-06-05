import { useCallback } from 'react';
import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { RoutePath } from '../../../../shared/routes';
import { IconButton, IconButtonProps, MainHeader } from '../../../lib/components';
import { Dot } from '../../../lib/components/dot';
import { TransitionType, useHistory } from '../../../lib/history';
import { useSelector } from '../../../redux/store';

export type MainHeaderSettingsButtonProps = Omit<IconButtonProps, 'icon'>;

const StyledDot = styled(Dot)`
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
      history.push(RoutePath.settings, { transition: TransitionType.show });
    }
  }, [history, props.disabled]);

  return (
    <MainHeader.IconButton onClick={openSettings} aria-label={messages.gettext('Settings')}>
      {suggestedUpgrade ? (
        <StyledDiv>
          <IconButton.Icon icon="settings-partial" />
          <StyledDot variant="warning" size="tiny" />
        </StyledDiv>
      ) : (
        <IconButton.Icon icon="settings-filled" />
      )}
    </MainHeader.IconButton>
  );
}
