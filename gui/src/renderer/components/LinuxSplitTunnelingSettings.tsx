import React, { useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react';
import { useHistory } from 'react-router';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import { ILinuxSplitTunnelingApplication } from '../../shared/application-types';
import consumePromise from '../../shared/promise';
import { useAppContext } from '../context';
import * as AppButton from './AppButton';
import * as Cell from './cell';
import ImageView from './ImageView';
import { Container, Layout } from './Layout';
import { ModalContainer, ModalAlert, ModalAlertType } from './Modal';
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

const StyledCellButton = styled(Cell.CellButton)((props: { lookDisabled: boolean }) => ({
  ':not(:disabled):hover': {
    backgroundColor: props.lookDisabled ? colors.blue : undefined,
  },
}));

const disabledApplication = (props: { lookDisabled: boolean }) => ({
  opacity: props.lookDisabled ? 0.6 : undefined,
});

const StyledIcon = styled(Cell.UntintedIcon)(disabledApplication, {
  marginRight: '12px',
});

const StyledCellLabel = styled(Cell.Label)(disabledApplication, {
  fontFamily: 'Open Sans',
  fontWeight: 'normal',
  fontSize: '16px',
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
  const {
    getSplitTunnelingApplications,
    launchExcludedApplication,
    showOpenDialog,
  } = useAppContext();
  const history = useHistory();

  const [applications, setApplications] = useState<ILinuxSplitTunnelingApplication[]>();
  const [applicationListHeight, setApplicationListHeight] = useState<number>();
  const [browsing, setBrowsing] = useState(false);
  const [browseError, setBrowseError] = useState<string>();

  const applicationListRef = useRef() as React.RefObject<HTMLDivElement>;

  const launchApplication = useCallback(
    async (application: ILinuxSplitTunnelingApplication | string) => {
      const result = await launchExcludedApplication(application);
      if ('error' in result) {
        setBrowseError(result.error);
      }
    },
    [],
  );

  const launchWithFilePicker = useCallback(async () => {
    setBrowsing(true);
    const file = await showOpenDialog({
      properties: ['openFile'],
      buttonLabel: messages.pgettext('split-tunneling-view', 'Launch application'),
    });
    setBrowsing(false);

    if (file.filePaths[0]) {
      await launchApplication(file.filePaths[0]);
    }
  }, []);

  const hideBrowseFailureDialog = useCallback(() => setBrowseError(undefined), []);

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
      <ModalContainer>
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
                            launchApplication={launchApplication}
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
        {browseError && (
          <ModalAlert
            type={ModalAlertType.warning}
            iconColor={colors.red}
            message={sprintf(
              // TRANSLATORS: Error message showed in a dialog when an application failes to launch.
              messages.pgettext(
                'split-tunneling-view',
                'Unable to launch selection. %(detailedErrorMessage)s',
              ),
              { detailedErrorMessage: browseError },
            )}
            buttons={[
              <AppButton.BlueButton key="close" onClick={hideBrowseFailureDialog}>
                {messages.gettext('Close')}
              </AppButton.BlueButton>,
            ]}
            close={hideBrowseFailureDialog}
          />
        )}
      </ModalContainer>
    </>
  );
}

interface IApplicationRowProps {
  application: ILinuxSplitTunnelingApplication;
  launchApplication: (application: ILinuxSplitTunnelingApplication) => void;
}

function ApplicationRow(props: IApplicationRowProps) {
  const [showWarning, setShowWarning] = useState(false);

  const launch = useCallback(() => {
    setShowWarning(false);
    props.launchApplication(props.application);
  }, [props.launchApplication, props.application]);

  const showWarningDialog = useCallback(() => setShowWarning(true), []);
  const hideWarningDialog = useCallback(() => setShowWarning(false), []);

  const disabled = props.application.warning === 'launches-elsewhere';
  const warningColor = disabled ? colors.red : colors.yellow;
  const warningMessage = disabled
    ? sprintf(
        messages.pgettext(
          'split-tunneling-view',
          '%(applicationName)s is problematic and can’t be excluded from the VPN tunnel.',
        ),
        {
          applicationName: props.application.name,
        },
      )
    : sprintf(
        messages.pgettext(
          'split-tunneling-view',
          'If it’s already running, close %(applicationName)s before launching it from here. Otherwise it might not be excluded from the VPN tunnel.',
        ),
        {
          applicationName: props.application.name,
        },
      );
  const warningDialogButtons = disabled
    ? [
        <AppButton.BlueButton key="cancel" onClick={hideWarningDialog}>
          {messages.gettext('Back')}
        </AppButton.BlueButton>,
      ]
    : [
        <AppButton.BlueButton key="launch" onClick={launch}>
          {messages.pgettext('split-tunneling-view', 'Launch')}
        </AppButton.BlueButton>,
        <AppButton.BlueButton key="cancel" onClick={hideWarningDialog}>
          {messages.gettext('Cancel')}
        </AppButton.BlueButton>,
      ];

  return (
    <>
      <StyledCellButton
        onClick={props.application.warning ? showWarningDialog : launch}
        lookDisabled={disabled}>
        {props.application.icon ? (
          <StyledIcon
            source={props.application.icon}
            width={35}
            height={35}
            lookDisabled={disabled}
          />
        ) : (
          <StyledIconPlaceholder />
        )}
        <StyledCellLabel lookDisabled={disabled}>{props.application.name}</StyledCellLabel>
        {props.application.warning && <Cell.Icon source="icon-alert" tintColor={warningColor} />}
      </StyledCellButton>
      {showWarning && (
        <ModalAlert
          type={ModalAlertType.warning}
          iconColor={warningColor}
          message={warningMessage}
          buttons={warningDialogButtons}
          close={hideWarningDialog}
        />
      )}
    </>
  );
}
