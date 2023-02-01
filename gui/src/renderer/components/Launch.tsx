import { useCallback } from 'react';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { useSelector } from '../redux/store';
import * as AppButton from './AppButton';
import { measurements, tinyText } from './common-styles';
import ErrorView from './ErrorView';
import { Footer } from './Layout';

export default function Launch() {
  const daemonAllowed = useSelector((state) => state.userInterface.daemonAllowed);
  const footer = <SettingsFooter show={daemonAllowed === false} />;

  return (
    <ErrorView footer={footer}>
      {messages.pgettext('launch-view', 'Connecting to Mullvad system service...')}
    </ErrorView>
  );
}

const StyledFooter = styled(Footer)({}, (props: { show: boolean }) => ({
  backgroundColor: colors.blue,
  padding: `0 14px ${measurements.viewMargin}`,
  opacity: props.show ? 1 : 0,
  transition: 'opacity 250ms ease-in-out',
}));

const StyledSystemSettingsContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  backgroundColor: colors.darkBlue,
  borderRadius: '8px',
  margin: 0,
  padding: '16px',
});

const StyledLaunchFooterPrompt = styled.span(tinyText, {
  color: colors.white,
  margin: `8px 0 ${measurements.buttonVerticalMargin} 0`,
});

interface ISettingsFooterProps {
  show: boolean;
}

function SettingsFooter(props: ISettingsFooterProps) {
  const { showLaunchDaemonSettings } = useAppContext();

  const openSettings = useCallback(async () => {
    await showLaunchDaemonSettings();
  }, []);

  return (
    <StyledFooter show={props.show}>
      <StyledSystemSettingsContainer>
        <StyledLaunchFooterPrompt>
          {messages.pgettext(
            'launch-view',
            'Permission for the Mullvad VPN service has been revoked. Please go to System Settings and allow Mullvad VPN under the “Allow in the Background” setting.',
          )}
        </StyledLaunchFooterPrompt>
        <AppButton.BlueButton onClick={openSettings}>
          {messages.gettext('Go to System Settings')}
        </AppButton.BlueButton>
      </StyledSystemSettingsContainer>
    </StyledFooter>
  );
}
