import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { useSelector } from 'react-redux';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import {
  IApplication,
  ILinuxSplitTunnelingApplication,
  ISplitTunnelingApplication,
} from '../../shared/application-types';
import { strings } from '../../shared/constants';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { Button, Container, Flex, FootnoteMini, IconButton, Spinner } from '../lib/components';
import { Colors } from '../lib/foundations';
import { useHistory } from '../lib/history';
import { formatHtml } from '../lib/html-formatter';
import { useAfterTransition } from '../lib/transition-hooks';
import { useEffectEvent, useStyledRef } from '../lib/utility-hooks';
import { IReduxState } from '../redux/store';
import { AppNavigationHeader } from './';
import Accordion from './Accordion';
import * as AppButton from './AppButton';
import * as Cell from './cell';
import { CustomScrollbarsRef } from './CustomScrollbars';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';
import List from './List';
import { ModalAlert, ModalAlertType } from './Modal';
import { NavigationContainer } from './NavigationContainer';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from './SettingsHeader';
import {
  StyledBrowseButton,
  StyledCellButton,
  StyledCellLabel,
  StyledCellWarningIcon,
  StyledIcon,
  StyledIconPlaceholder,
  StyledNavigationScrollbars,
  StyledNoResult,
  StyledNoResultText,
  StyledPageCover,
  StyledSearchBar,
  StyledSpinnerRow,
  WideSmallButton,
} from './SplitTunnelingSettingsStyles';
import Switch from './Switch';

export default function SplitTunneling() {
  const { pop } = useHistory();
  const [browsing, setBrowsing] = useState(false);
  const scrollbarsRef = useStyledRef<CustomScrollbarsRef>();

  const scrollToTop = useCallback(() => scrollbarsRef.current?.scrollToTop(true), [scrollbarsRef]);

  return (
    <>
      <StyledPageCover $show={browsing} />
      <BackAction action={pop}>
        <Layout>
          <SettingsContainer>
            <NavigationContainer>
              <AppNavigationHeader title={strings.splitTunneling} />

              <StyledNavigationScrollbars ref={scrollbarsRef}>
                <PlatformSpecificSplitTunnelingSettings
                  setBrowsing={setBrowsing}
                  scrollToTop={scrollToTop}
                />
              </StyledNavigationScrollbars>
            </NavigationContainer>
          </SettingsContainer>
        </Layout>
      </BackAction>
    </>
  );
}

interface IPlatformSplitTunnelingSettingsProps {
  setBrowsing: (value: boolean) => void;
  scrollToTop: () => void;
}

function PlatformSpecificSplitTunnelingSettings(props: IPlatformSplitTunnelingSettingsProps) {
  switch (window.env.platform) {
    case 'linux':
      return <LinuxSplitTunnelingSettings {...props} />;
    default:
      return <SplitTunnelingSettings {...props} />;
  }
}

function useFilePicker(
  buttonLabel: string,
  setOpen: (value: boolean) => void,
  select: (path: string) => void,
  filter?: { name: string; extensions: string[] },
) {
  const { showOpenDialog } = useAppContext();

  return useCallback(async () => {
    setOpen(true);
    const file = await showOpenDialog({
      properties: ['openFile'],
      buttonLabel,
      filters: filter ? [filter] : undefined,
    });
    setOpen(false);

    if (file.filePaths[0]) {
      select(file.filePaths[0]);
    }
  }, [setOpen, showOpenDialog, buttonLabel, filter, select]);
}

function LinuxSplitTunnelingSettings(props: IPlatformSplitTunnelingSettingsProps) {
  const { getLinuxSplitTunnelingApplications, launchExcludedApplication } = useAppContext();
  const runAfterTransition = useAfterTransition();

  const [searchTerm, setSearchTerm] = useState('');
  const [applications, setApplications] = useState<ILinuxSplitTunnelingApplication[]>();
  const [browseError, setBrowseError] = useState<string>();

  const updateApplications = useEffectEvent(() => {
    runAfterTransition(async () => {
      const applications = await getLinuxSplitTunnelingApplications();
      setApplications(applications);
    });
  });

  useEffect(() => void updateApplications(), []);

  const launchApplication = useCallback(
    async (application: ILinuxSplitTunnelingApplication | string) => {
      const result = await launchExcludedApplication(application);
      if ('error' in result) {
        setBrowseError(result.error);
      }
    },
    [launchExcludedApplication],
  );

  const launchWithFilePicker = useFilePicker(
    messages.pgettext('split-tunneling-view', 'Launch'),
    props.setBrowsing,
    launchApplication,
  );

  const filteredApplications = useMemo(
    () => applications?.filter((application) => includesSearchTerm(application, searchTerm)),
    [applications, searchTerm],
  );

  const hideBrowseFailureDialog = useCallback(() => setBrowseError(undefined), []);

  const rowRenderer = useCallback(
    (application: ILinuxSplitTunnelingApplication) => (
      <LinuxApplicationRow application={application} onSelect={launchApplication} />
    ),
    [launchApplication],
  );

  return (
    <>
      <SettingsHeader>
        <HeaderTitle>{strings.splitTunneling}</HeaderTitle>
        <HeaderSubTitle>
          {messages.pgettext(
            'split-tunneling-view',
            'Click on an app to launch it. Its traffic will bypass the VPN tunnel until you close it.',
          )}
        </HeaderSubTitle>
      </SettingsHeader>

      <StyledSearchBar searchTerm={searchTerm} onSearch={setSearchTerm} />

      {searchTerm !== '' &&
        (filteredApplications === undefined || filteredApplications.length === 0) && (
          <StyledNoResult>
            <StyledNoResultText>
              {formatHtml(
                sprintf(messages.gettext('No result for <b>%(searchTerm)s</b>.'), { searchTerm }),
              )}
            </StyledNoResultText>
            <StyledNoResultText>{messages.gettext('Try a different search.')}</StyledNoResultText>
          </StyledNoResult>
        )}

      <Flex $flexDirection="column" $gap="medium">
        {filteredApplications !== undefined && filteredApplications.length > 0 && (
          <ApplicationList applications={filteredApplications} rowRenderer={rowRenderer} />
        )}

        <StyledBrowseButton onClick={launchWithFilePicker}>
          {messages.pgettext('split-tunneling-view', 'Find another app')}
        </StyledBrowseButton>
      </Flex>

      <ModalAlert
        isOpen={browseError !== undefined}
        type={ModalAlertType.warning}
        iconColor={Colors.red}
        message={sprintf(
          // TRANSLATORS: Error message showed in a dialog when an application fails to launch.
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
    </>
  );
}

interface ILinuxApplicationRowProps {
  application: ILinuxSplitTunnelingApplication;
  onSelect?: (application: ILinuxSplitTunnelingApplication) => void;
}

function LinuxApplicationRow(props: ILinuxApplicationRowProps) {
  const { onSelect } = props;

  const [showWarning, setShowWarning] = useState(false);

  const launch = useCallback(() => {
    setShowWarning(false);
    onSelect?.(props.application);
  }, [onSelect, props.application]);

  const showWarningDialog = useCallback(() => setShowWarning(true), []);
  const hideWarningDialog = useCallback(() => setShowWarning(false), []);

  const disabled = props.application.warning === 'launches-elsewhere';
  const warningColor = disabled ? Colors.red : Colors.yellow;
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
        $lookDisabled={disabled}>
        {props.application.icon ? (
          <StyledIcon
            source={props.application.icon}
            width={35}
            height={35}
            $lookDisabled={disabled}
          />
        ) : (
          <StyledIconPlaceholder />
        )}
        <StyledCellLabel $lookDisabled={disabled}>{props.application.name}</StyledCellLabel>
        {props.application.warning && (
          <StyledCellWarningIcon icon="alert-circle" color={warningColor} />
        )}
      </StyledCellButton>
      <ModalAlert
        isOpen={showWarning}
        type={ModalAlertType.warning}
        iconColor={warningColor}
        message={warningMessage}
        buttons={warningDialogButtons}
        close={hideWarningDialog}
      />
    </>
  );
}

export function SplitTunnelingSettings(props: IPlatformSplitTunnelingSettingsProps) {
  const { scrollToTop } = props;

  const {
    addSplitTunnelingApplication,
    removeSplitTunnelingApplication,
    forgetManuallyAddedSplitTunnelingApplication,
    getSplitTunnelingApplications,
    needFullDiskPermissions,
    setSplitTunnelingState,
  } = useAppContext();
  const runAfterTransition = useAfterTransition();
  const splitTunnelingEnabled = useSelector((state: IReduxState) => state.settings.splitTunneling);
  const splitTunnelingApplications = useSelector(
    (state: IReduxState) => state.settings.splitTunnelingApplications,
  );

  const [searchTerm, setSearchTerm] = useState('');
  const [applications, setApplications] = useState<ISplitTunnelingApplication[]>();

  const [loadingDiskPermissions, setLoadingDiskPermissions] = useState(false);
  const [splitTunnelingAvailable, setSplitTunnelingAvailable] = useState(
    window.env.platform === 'darwin' ? undefined : true,
  );

  const canEditSplitTunneling = splitTunnelingEnabled && (splitTunnelingAvailable ?? false);

  const fetchNeedFullDiskPermissions = useCallback(async () => {
    setLoadingDiskPermissions(true);
    const needPermissions = await needFullDiskPermissions();
    setSplitTunnelingAvailable(!needPermissions);
    setLoadingDiskPermissions(false);
  }, [needFullDiskPermissions]);

  useEffect((): void | (() => void) => {
    if (window.env.platform === 'darwin') {
      void fetchNeedFullDiskPermissions();
    }
  }, [fetchNeedFullDiskPermissions]);

  const onMount = useEffectEvent(() => {
    runAfterTransition(async () => {
      const { fromCache, applications } = await getSplitTunnelingApplications();
      setApplications(applications);

      if (fromCache) {
        const { applications } = await getSplitTunnelingApplications(true);
        setApplications(applications);
      }
    });
  });

  useEffect(() => void onMount(), []);

  const filteredSplitApplications = useMemo(
    () =>
      splitTunnelingApplications.filter((application) =>
        includesSearchTerm(application, searchTerm),
      ),
    [splitTunnelingApplications, searchTerm],
  );

  const filteredNonSplitApplications = useMemo(() => {
    return applications?.filter(
      (application) =>
        includesSearchTerm(application, searchTerm) &&
        !splitTunnelingApplications.some(
          (splitTunnelingApplication) =>
            application.absolutepath === splitTunnelingApplication.absolutepath,
        ),
    );
  }, [applications, splitTunnelingApplications, searchTerm]);

  const addApplication = useCallback(
    async (application: ISplitTunnelingApplication | string) => {
      if (!canEditSplitTunneling) {
        await setSplitTunnelingState(true);
      }
      await addSplitTunnelingApplication(application);
    },
    [addSplitTunnelingApplication, canEditSplitTunneling, setSplitTunnelingState],
  );

  const addBrowsedForApplication = useCallback(
    async (application: string) => {
      await addApplication(application);
      const { applications } = await getSplitTunnelingApplications();
      setApplications(applications);
    },
    [addApplication, getSplitTunnelingApplications],
  );

  const forgetManuallyAddedApplicationAndUpdate = useCallback(
    async (application: ISplitTunnelingApplication) => {
      await forgetManuallyAddedSplitTunnelingApplication(application);
      const { applications } = await getSplitTunnelingApplications();
      setApplications(applications);
    },
    [forgetManuallyAddedSplitTunnelingApplication, getSplitTunnelingApplications],
  );

  const removeApplication = useCallback(
    async (application: ISplitTunnelingApplication) => {
      if (!canEditSplitTunneling) {
        await setSplitTunnelingState(true);
      }
      removeSplitTunnelingApplication(application);
    },
    [removeSplitTunnelingApplication, setSplitTunnelingState, canEditSplitTunneling],
  );

  const filePickerCallback = useFilePicker(
    messages.pgettext('split-tunneling-view', 'Add'),
    props.setBrowsing,
    addBrowsedForApplication,
    getFilePickerOptionsForPlatform(),
  );

  const addWithFilePicker = useCallback(async () => {
    scrollToTop();
    await filePickerCallback();
  }, [filePickerCallback, scrollToTop]);

  const excludedRowRenderer = useCallback(
    (application: ISplitTunnelingApplication) => (
      <ApplicationRow application={application} onRemove={removeApplication} />
    ),
    [removeApplication],
  );

  const includedRowRenderer = useCallback(
    (application: ISplitTunnelingApplication) => {
      const onForget = application.deletable ? forgetManuallyAddedApplicationAndUpdate : undefined;
      return (
        <ApplicationRow application={application} onAdd={addApplication} onDelete={onForget} />
      );
    },
    [addApplication, forgetManuallyAddedApplicationAndUpdate],
  );

  const showSplitSection = canEditSplitTunneling && filteredSplitApplications.length > 0;
  const showNonSplitSection =
    canEditSplitTunneling &&
    (!filteredNonSplitApplications || filteredNonSplitApplications.length > 0);

  const excludedTitle = (
    <Cell.SectionTitle>
      {messages.pgettext('split-tunneling-view', 'Excluded apps')}
    </Cell.SectionTitle>
  );

  const allTitle = (
    <Cell.SectionTitle>{messages.pgettext('split-tunneling-view', 'All apps')}</Cell.SectionTitle>
  );

  return (
    <>
      <SettingsHeader>
        <Flex $justifyContent="space-between" $alignItems="center">
          <HeaderTitle>{strings.splitTunneling}</HeaderTitle>
          <Switch
            isOn={splitTunnelingEnabled}
            disabled={
              !splitTunnelingEnabled && (!splitTunnelingAvailable || loadingDiskPermissions)
            }
            onChange={setSplitTunnelingState}
          />
        </Flex>
        {!loadingDiskPermissions && (
          <>
            <MacOsSplitTunnelingAvailability
              needFullDiskPermissions={
                window.env.platform === 'darwin' && splitTunnelingAvailable === false
              }
            />
            {splitTunnelingAvailable && (
              <HeaderSubTitle>
                {messages.pgettext(
                  'split-tunneling-view',
                  'Choose the apps you want to exclude from the VPN tunnel.',
                )}
              </HeaderSubTitle>
            )}
          </>
        )}
      </SettingsHeader>
      {loadingDiskPermissions && (
        <Flex $justifyContent="center" $margin={{ top: 'large' }}>
          <Spinner size="big" />
        </Flex>
      )}

      {canEditSplitTunneling && (
        <StyledSearchBar searchTerm={searchTerm} onSearch={setSearchTerm} />
      )}

      {canEditSplitTunneling && searchTerm !== '' && !showSplitSection && !showNonSplitSection && (
        <StyledNoResult>
          <StyledNoResultText>
            {formatHtml(
              sprintf(messages.gettext('No result for <b>%(searchTerm)s</b>.'), { searchTerm }),
            )}
          </StyledNoResultText>
          <StyledNoResultText>{messages.gettext('Try a different search.')}</StyledNoResultText>
        </StyledNoResult>
      )}

      <Flex $flexDirection="column" $gap="medium" $margin={{ bottom: 'large' }}>
        {(showSplitSection || showNonSplitSection) && (
          <Flex $flexDirection="column" $gap="medium">
            <Accordion expanded={showSplitSection}>
              <Cell.Section sectionTitle={excludedTitle}>
                <ApplicationList
                  data-testid="split-applications"
                  applications={filteredSplitApplications}
                  rowRenderer={excludedRowRenderer}
                />
              </Cell.Section>
            </Accordion>

            <Accordion expanded={showNonSplitSection}>
              <Cell.Section sectionTitle={allTitle}>
                <ApplicationList
                  data-testid="non-split-applications"
                  applications={filteredNonSplitApplications}
                  rowRenderer={includedRowRenderer}
                />
              </Cell.Section>
            </Accordion>
          </Flex>
        )}

        {canEditSplitTunneling && (
          <Container size="3">
            <Button onClick={addWithFilePicker}>
              <Button.Text>
                {messages.pgettext('split-tunneling-view', 'Find another app')}
              </Button.Text>
            </Button>
          </Container>
        )}
      </Flex>
    </>
  );
}

interface MacOsSplitTunnelingAvailabilityProps {
  needFullDiskPermissions: boolean;
}

function MacOsSplitTunnelingAvailability({
  needFullDiskPermissions,
}: MacOsSplitTunnelingAvailabilityProps) {
  const { showFullDiskAccessSettings, daemonPrepareRestart } = useAppContext();
  const restartDaemon = useCallback(() => daemonPrepareRestart(true), [daemonPrepareRestart]);

  if (!needFullDiskPermissions) return null;

  return (
    <Flex $flexDirection="column" $gap="large">
      <HeaderSubTitle>
        {messages.pgettext(
          'split-tunneling-view',
          'To use split tunneling please enable “Full disk access” for “Mullvad VPN” in the macOS system settings.',
        )}
      </HeaderSubTitle>
      <Flex $flexDirection="column" $gap="small">
        <Flex $flexDirection="column" $gap="big">
          <WideSmallButton onClick={showFullDiskAccessSettings}>
            {messages.pgettext('split-tunneling-view', 'Open System Settings')}
          </WideSmallButton>
          <FootnoteMini color={Colors.white60}>
            {messages.pgettext(
              'split-tunneling-view',
              'Enabled "Full disk access" and still having issues?',
            )}
          </FootnoteMini>
        </Flex>
        <WideSmallButton onClick={restartDaemon}>
          {messages.pgettext('split-tunneling-view', 'Restart Mullvad Service')}
        </WideSmallButton>
      </Flex>
    </Flex>
  );
}

interface IApplicationListProps<T extends IApplication> {
  applications: T[] | undefined;
  rowRenderer: (application: T) => React.ReactElement;
  'data-testid'?: string;
}

function ApplicationList<T extends IApplication>(props: IApplicationListProps<T>) {
  if (props.applications == undefined) {
    return (
      <StyledSpinnerRow>
        <Spinner size="big" />
      </StyledSpinnerRow>
    );
  } else {
    return (
      <Flex $flexDirection="column" data-testid={props['data-testid']}>
        <List
          data-testid={props['data-testid']}
          items={props.applications.sort((a, b) => a.name.localeCompare(b.name))}
          getKey={applicationGetKey}>
          {props.rowRenderer}
        </List>
      </Flex>
    );
  }
}

function applicationGetKey<T extends IApplication>(application: T): string {
  return application.absolutepath;
}

const StyledContainer = styled(Cell.Container)({
  backgroundColor: Colors.blue40,
});

interface IApplicationRowProps {
  application: ISplitTunnelingApplication;
  onAdd?: (application: ISplitTunnelingApplication) => void;
  onRemove?: (application: ISplitTunnelingApplication) => void;
  onDelete?: (application: ISplitTunnelingApplication) => void;
}

function ApplicationRow(props: IApplicationRowProps) {
  const { onAdd: propsOnAdd, onRemove: propsOnRemove, onDelete: propsOnDelete } = props;

  const onAdd = useCallback(() => {
    propsOnAdd?.(props.application);
  }, [propsOnAdd, props.application]);

  const onRemove = useCallback(() => {
    propsOnRemove?.(props.application);
  }, [propsOnRemove, props.application]);

  const onDelete = useCallback(() => {
    propsOnDelete?.(props.application);
  }, [propsOnDelete, props.application]);

  return (
    <StyledContainer>
      {props.application.icon ? (
        <StyledIcon source={props.application.icon} width={35} height={35} />
      ) : (
        <StyledIconPlaceholder />
      )}
      <StyledCellLabel>{props.application.name}</StyledCellLabel>
      <Flex $gap="small">
        {props.onDelete && (
          <IconButton variant="secondary" onClick={onDelete}>
            <IconButton.Icon icon="cross-circle" />
          </IconButton>
        )}
        {props.onAdd && (
          <IconButton variant="secondary" onClick={onAdd}>
            <IconButton.Icon icon="add-circle" />
          </IconButton>
        )}
        {props.onRemove && (
          <IconButton variant="secondary" onClick={onRemove}>
            <IconButton.Icon icon="remove-circle" />
          </IconButton>
        )}
      </Flex>
    </StyledContainer>
  );
}

function includesSearchTerm(application: IApplication, searchTerm: string) {
  return application.name.toLowerCase().includes(searchTerm.toLowerCase());
}

function getFilePickerOptionsForPlatform():
  | { name: string; extensions: Array<string> }
  | undefined {
  return window.env.platform === 'win32'
    ? { name: 'Executables', extensions: ['exe', 'lnk'] }
    : undefined;
}
