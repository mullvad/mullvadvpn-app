import { remote } from 'electron';
import React, { useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react';
import { useHistory } from 'react-router';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import ISplitTunnelingApplication from '../../shared/linux-split-tunneling-application';
import consumePromise from '../../shared/promise';
import { useAppContext } from '../context';
import * as AppButton from './AppButton';
import * as Cell from './Cell';
import ImageView from './ImageView';
import { Container, Layout } from './Layout';
import {
  BackBarItem,
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from './SettingsHeader';

const StyledPageCover = styled.div({}, (props: { show: boolean }) => ({
  position: 'absolute',
  zIndex: 2,
  top: 0,
  left: 0,
  right: 0,
  bottom: 0,
  backgroundColor: '#000000',
  opacity: 0.6,
  display: props.show ? 'block' : 'none',
}));

const StyledContainer = styled(Container)({
  backgroundColor: colors.darkBlue,
});

const StyledNavigationScrollbars = styled(NavigationScrollbars)({
  flex: 1,
});

const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
});

const StyledIcon = styled(Cell.UntintedIcon)({
  marginRight: '12px',
});

const StyledIconPlaceholder = styled.div({
  width: '35px',
  marginRight: '12px',
});

const StyledApplicationListContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
});

const StyledApplicationListAnimation = styled.div({}, (props: { height?: number }) => ({
  overflow: 'hidden',
  height: props.height ? `${props.height}px` : 'auto',
  transition: 'height 500ms ease-in-out',
  marginBottom: '20px',
}));

const StyledSpinnerRow = styled.div({
  display: 'flex',
  justifyContent: 'center',
  padding: '8px 0',
  background: colors.blue40,
});

const StyledBrowseButton = styled(AppButton.BlueButton)({
  margin: '0 22px 22px',
});

export default function LinuxSplitTunnelingSettings() {
  const { getSplitTunnelingApplications, launchExcludedApplication } = useAppContext();
  const history = useHistory();

  const [applications, setApplications] = useState<ISplitTunnelingApplication[]>();
  const [applicationListHeight, setApplicationListHeight] = useState<number>();
  const [browsing, setBrowsing] = useState(false);

  const applicationListRef = useRef() as React.RefObject<HTMLDivElement>;

  const launchWithFilePicker = useCallback(async () => {
    setBrowsing(true);
    const file = await remote.dialog.showOpenDialog({
      properties: ['openFile'],
      buttonLabel: messages.pgettext('split-tunneling-view', 'Launch application'),
    });
    setBrowsing(false);

    if (file.filePaths[0]) {
      launchExcludedApplication(file.filePaths[0]);
    }
  }, []);

  useEffect(() => {
    consumePromise(getSplitTunnelingApplications().then(setApplications));
  }, []);

  useLayoutEffect(() => {
    const height = applicationListRef.current?.getBoundingClientRect().height;
    setApplicationListHeight(height);
  }, [applications]);

  return (
    <>
      <StyledPageCover show={browsing} />
      <Layout>
        <StyledContainer>
          <NavigationContainer>
            <NavigationBar>
              <NavigationItems>
                <BackBarItem action={history.goBack}>
                  {
                    // TRANSLATORS: Back button in navigation bar
                    messages.pgettext('navigation-bar', 'Advanced')
                  }
                </BackBarItem>
                <TitleBarItem>
                  {
                    // TRANSLATORS: Title label in navigation bar
                    messages.pgettext('split-tunneling-nav', 'Split tunneling')
                  }
                </TitleBarItem>
              </NavigationItems>
            </NavigationBar>

            <StyledNavigationScrollbars>
              <StyledContent>
                <SettingsHeader>
                  <HeaderTitle>
                    {messages.pgettext('split-tunneling-view', 'Split tunneling')}
                  </HeaderTitle>
                  <HeaderSubTitle>
                    {messages.pgettext(
                      'split-tunneling-view',
                      'Click on an app to launch it. Its traffic will bypass the VPN tunnel until you close it.',
                    )}
                  </HeaderSubTitle>
                </SettingsHeader>

                <StyledApplicationListAnimation height={applicationListHeight}>
                  <StyledApplicationListContent ref={applicationListRef}>
                    {applications === undefined ? (
                      <StyledSpinnerRow>
                        <ImageView source="icon-spinner" height={60} width={60} />
                      </StyledSpinnerRow>
                    ) : (
                      applications.map((application) => (
                        <ApplicationRow
                          key={application.absolutepath}
                          application={application}
                          launchApplication={launchExcludedApplication}
                        />
                      ))
                    )}
                  </StyledApplicationListContent>
                </StyledApplicationListAnimation>

                <StyledBrowseButton onClick={launchWithFilePicker}>
                  {messages.pgettext('split-tunneling-view', 'Browse')}
                </StyledBrowseButton>
              </StyledContent>
            </StyledNavigationScrollbars>
          </NavigationContainer>
        </StyledContainer>
      </Layout>
    </>
  );
}

interface IApplicationRowProps {
  application: ISplitTunnelingApplication;
  launchApplication: (application: ISplitTunnelingApplication) => void;
}

function ApplicationRow(props: IApplicationRowProps) {
  const onClick = useCallback(() => {
    props.launchApplication(props.application);
  }, [props.launchApplication, props.application]);

  return (
    <Cell.CellButton onClick={onClick}>
      {props.application.icon ? (
        <StyledIcon source={props.application.icon} width={35} height={35} />
      ) : (
        <StyledIconPlaceholder />
      )}
      <Cell.Label>{props.application.name}</Cell.Label>
    </Cell.CellButton>
  );
}
