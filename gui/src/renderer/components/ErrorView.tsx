import React, { useCallback } from 'react';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import * as AppButton from './AppButton';
import { measurements, tinyText } from './common-styles';
import { HeaderBarSettingsButton } from './HeaderBar';
import ImageView from './ImageView';
import { Container, Footer, Header, Layout } from './Layout';

const StyledContainer = styled(Container)({
  flex: 1,
  flexDirection: 'column',
  alignItems: 'center',
  justifyContent: 'center',
  marginTop: '-150px',
});

const StyledFooter = styled(Footer)({}, (props: { show: boolean }) => ({
  backgroundColor: colors.blue,
  padding: '12px',
  position: 'absolute',
  bottom: 0,
  transform: `translateY(${props.show ? 0 : 100}%)`,
  transition: 'transform 250ms ease-in-out',
}));

const StyledSystemSettingsContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  alignSelf: 'start',
  backgroundColor: colors.darkBlue,
  borderRadius: '8px',
  margin: 0,
  padding: '16px',
});

const StyledLaunchFooterPrompt = styled.span(tinyText, {
  color: colors.white,
  margin: '9px 0 20px 0',
});

const Logo = styled(ImageView)({
  marginBottom: '12px',
});

const Title = styled(ImageView)({
  opacity: 0.6,
  marginBottom: '9px',
});

const Subtitle = styled.span({
  fontFamily: 'Open Sans',
  fontSize: '14px',
  lineHeight: '20px',
  margin: `0 ${measurements.viewMargin}`,
  color: colors.white40,
  textAlign: 'center',
});

interface ErrorViewProps {
  settingsUnavailable?: boolean;
  showSettingsFooter?: boolean;
  children: React.ReactNode | React.ReactNode[];
}

export default class ErrorView extends React.Component<ErrorViewProps> {
  public render() {
    return (
      <Layout>
        <Header>{!this.props.settingsUnavailable && <HeaderBarSettingsButton />}</Header>
        <StyledContainer>
          <Logo height={106} width={106} source="logo-icon" />
          <Title height={18} source="logo-text" />
          <Subtitle role="alert">{this.props.children}</Subtitle>
        </StyledContainer>
        <SettingsFooter show={this.props.showSettingsFooter} />
      </Layout>
    );
  }
}

interface ISettingsFooterProps {
  show?: boolean;
}

function SettingsFooter(props: ISettingsFooterProps) {
  const { showLaunchDaemonSettings } = useAppContext();

  const openSettings = useCallback(async () => {
    await showLaunchDaemonSettings();
  }, []);

  return (
    <StyledFooter show={props.show ? props.show : false}>
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
