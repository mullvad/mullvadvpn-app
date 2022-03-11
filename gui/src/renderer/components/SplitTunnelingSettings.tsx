import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useSelector } from 'react-redux';
import { sprintf } from 'sprintf-js';
import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import { IApplication, ILinuxSplitTunnelingApplication } from '../../shared/application-types';
import { useAppContext } from '../context';
import { useHistory } from '../lib/history';
import { useAsyncEffect } from '../lib/utilityHooks';
import { IReduxState } from '../redux/store';
import Accordion from './Accordion';
import * as AppButton from './AppButton';
import * as Cell from './cell';
import { CustomScrollbarsRef } from './CustomScrollbars';
import ImageView from './ImageView';
import { Layout } from './Layout';
import List from './List';
import { ModalAlert, ModalAlertType } from './Modal';
import { NavigationBar, NavigationContainer, NavigationItems, TitleBarItem } from './NavigationBar';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from './SettingsHeader';
import {
  StyledPageCover,
  StyledContainer,
  StyledNavigationScrollbars,
  StyledContent,
  StyledCellButton,
  StyledIcon,
  StyledCellLabel,
  StyledIconPlaceholder,
  StyledSpinnerRow,
  StyledBrowseButton,
  StyledSearchInput,
  StyledClearButton,
  StyledSearchIcon,
  StyledClearIcon,
  StyledNoResultText,
  StyledSearchContainer,
  StyledNoResult,
  StyledActionIcon,
  StyledCellWarningIcon,
  StyledListContainer,
  StyledHeaderTitleContainer,
  StyledHeaderTitle,
} from './SplitTunnelingSettingsStyles';
import { formatMarkdown } from '../markdown-formatter';
import { BackAction } from './KeyboardNavigation';
import Switch from './Switch';

export default function SplitTunneling() {
  const { pop } = useHistory();
  const [browsing, setBrowsing] = useState(false);
  const scrollbarsRef = useRef() as React.RefObject<CustomScrollbarsRef>;

  const scrollToTop = useCallback(() => scrollbarsRef.current?.scrollToTop(true), [scrollbarsRef]);

  return (
    <>
      <StyledPageCover show={browsing} />
      <BackAction action={pop}>
        <Layout>
          <StyledContainer>
            <NavigationContainer>
              <NavigationBar>
                <NavigationItems>
                  <TitleBarItem>Split tunneling</TitleBarItem>
                </NavigationItems>
              </NavigationBar>

              <StyledNavigationScrollbars ref={scrollbarsRef}>
                <StyledContent>
                  <PlatformSpecificSplitTunnelingSettings
                    setBrowsing={setBrowsing}
                    scrollToTop={scrollToTop}
                  />
                </StyledContent>
              </StyledNavigationScrollbars>
            </NavigationContainer>
          </StyledContainer>
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
    case 'win32':
      return <WindowsSplitTunnelingSettings {...props} />;
    default:
      throw new Error(`Split tunneling not implemented on ${window.env.platform}`);
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
  }, [buttonLabel, setOpen, select]);
}

function LinuxSplitTunnelingSettings(props: IPlatformSplitTunnelingSettingsProps) {
  const { getLinuxSplitTunnelingApplications, launchExcludedApplication } = useAppContext();

  const [searchTerm, setSearchTerm] = useState('');
  const [applications, setApplications] = useState<ILinuxSplitTunnelingApplication[]>();
  const [browseError, setBrowseError] = useState<string>();

  useEffect(() => void getLinuxSplitTunnelingApplications().then(setApplications), []);

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

  return (
    <>
      <SettingsHeader>
        <HeaderTitle>Split tunneling</HeaderTitle>
        <HeaderSubTitle>
          {messages.pgettext(
            'split-tunneling-view',
            'Click on an app to launch it. Its traffic will bypass the VPN tunnel until you close it.',
          )}
        </HeaderSubTitle>
      </SettingsHeader>

      <SearchBar searchTerm={searchTerm} onSearch={setSearchTerm} />
      <ApplicationList
        applications={filteredApplications}
        onSelect={launchApplication}
        rowComponent={LinuxApplicationRow}
      />

      <StyledBrowseButton onClick={launchWithFilePicker}>
        {messages.pgettext('split-tunneling-view', 'Find another app')}
      </StyledBrowseButton>

      <ModalAlert
        isOpen={browseError !== undefined}
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
    </>
  );
}

interface ILinuxApplicationRowProps {
  application: ILinuxSplitTunnelingApplication;
  onSelect?: (application: ILinuxSplitTunnelingApplication) => void;
}

function LinuxApplicationRow(props: ILinuxApplicationRowProps) {
  const [showWarning, setShowWarning] = useState(false);

  const launch = useCallback(() => {
    setShowWarning(false);
    props.onSelect?.(props.application);
  }, [props.onSelect, props.application]);

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
        {props.application.warning && (
          <StyledCellWarningIcon source="icon-alert" tintColor={warningColor} width={18} />
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

export function WindowsSplitTunnelingSettings(props: IPlatformSplitTunnelingSettingsProps) {
  const {
    addSplitTunnelingApplication,
    removeSplitTunnelingApplication,
    getWindowsSplitTunnelingApplications,
    setSplitTunnelingState,
  } = useAppContext();
  const splitTunnelingEnabled = useSelector((state: IReduxState) => state.settings.splitTunneling);
  const splitTunnelingApplications = useSelector(
    (state: IReduxState) => state.settings.splitTunnelingApplications,
  );

  const [searchTerm, setSearchTerm] = useState('');
  const [applications, setApplications] = useState<IApplication[]>();
  useAsyncEffect(async () => {
    const { fromCache, applications } = await getWindowsSplitTunnelingApplications();
    setApplications(applications);

    if (fromCache) {
      const { applications } = await getWindowsSplitTunnelingApplications(true);
      setApplications(applications);
    }
  }, []);

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
    async (application: IApplication | string) => {
      if (!splitTunnelingEnabled) {
        await setSplitTunnelingState(true);
      }
      await addSplitTunnelingApplication(application);
    },
    [addSplitTunnelingApplication, splitTunnelingEnabled, setSplitTunnelingState],
  );

  const addApplicationAndUpdate = useCallback(
    async (application: IApplication | string) => {
      await addApplication(application);
      const { applications } = await getWindowsSplitTunnelingApplications();
      setApplications(applications);
    },
    [addApplication, getWindowsSplitTunnelingApplications],
  );

  const removeApplication = useCallback(
    async (application: IApplication) => {
      if (!splitTunnelingEnabled) {
        await setSplitTunnelingState(true);
      }
      removeSplitTunnelingApplication(application);
    },
    [removeSplitTunnelingApplication, splitTunnelingEnabled],
  );

  const filePickerCallback = useFilePicker(
    messages.pgettext('split-tunneling-view', 'Add'),
    props.setBrowsing,
    addApplicationAndUpdate,
    { name: 'Executables', extensions: ['exe', 'lnk'] },
  );

  const addWithFilePicker = useCallback(async () => {
    props.scrollToTop();
    await filePickerCallback();
  }, [filePickerCallback, props.scrollToTop]);

  const showSplitSection = splitTunnelingEnabled && filteredSplitApplications.length > 0;
  const showNonSplitSection =
    splitTunnelingEnabled &&
    (!filteredNonSplitApplications || filteredNonSplitApplications.length > 0);

  return (
    <>
      <SettingsHeader>
        <StyledHeaderTitleContainer>
          <StyledHeaderTitle>Split tunneling</StyledHeaderTitle>
          <Switch isOn={splitTunnelingEnabled} onChange={setSplitTunnelingState} />
        </StyledHeaderTitleContainer>
        <HeaderSubTitle>
          {messages.pgettext(
            'split-tunneling-view',
            'Choose the apps you want to exclude from the VPN tunnel.',
          )}
        </HeaderSubTitle>
      </SettingsHeader>

      {splitTunnelingEnabled && <SearchBar searchTerm={searchTerm} onSearch={setSearchTerm} />}

      <Accordion expanded={showSplitSection}>
        <Cell.Section>
          <Cell.SectionTitle>
            {messages.pgettext('split-tunneling-view', 'Excluded apps')}
          </Cell.SectionTitle>
          <ApplicationList
            applications={filteredSplitApplications}
            onRemove={removeApplication}
            rowComponent={ApplicationRow}
          />
        </Cell.Section>
      </Accordion>

      <Accordion expanded={showNonSplitSection}>
        <Cell.Section>
          <Cell.SectionTitle>
            {messages.pgettext('split-tunneling-view', 'All apps')}
          </Cell.SectionTitle>
          <ApplicationList
            applications={filteredNonSplitApplications}
            onSelect={addApplication}
            rowComponent={ApplicationRow}
          />
        </Cell.Section>
      </Accordion>

      {splitTunnelingEnabled && searchTerm !== '' && !showSplitSection && !showNonSplitSection && (
        <StyledNoResult>
          <StyledNoResultText>
            {formatMarkdown(
              sprintf(
                messages.pgettext('split-tunneling-view', 'No result for **%(searchTerm)s**.'),
                { searchTerm },
              ),
            )}
          </StyledNoResultText>
          <StyledNoResultText>
            {messages.pgettext('split-tunneling-view', 'Try a different search.')}
          </StyledNoResultText>
        </StyledNoResult>
      )}

      {splitTunnelingEnabled && (
        <StyledBrowseButton onClick={addWithFilePicker}>
          {messages.pgettext('split-tunneling-view', 'Find another app')}
        </StyledBrowseButton>
      )}
    </>
  );
}

interface IApplicationListProps<T extends IApplication> {
  applications: T[] | undefined;
  onSelect?: (application: T) => void;
  onRemove?: (application: T) => void;
  rowComponent: React.ComponentType<IApplicationRowProps<T>>;
}

function ApplicationList<T extends IApplication>(props: IApplicationListProps<T>) {
  if (props.applications === undefined) {
    return (
      <StyledSpinnerRow>
        <ImageView source="icon-spinner" height={60} width={60} />
      </StyledSpinnerRow>
    );
  } else {
    return (
      <StyledListContainer>
        <List items={props.applications} getKey={applicationGetKey}>
          {(application) => (
            <props.rowComponent
              key={application.absolutepath}
              application={application}
              onSelect={props.onSelect}
              onRemove={props.onRemove}
            />
          )}
        </List>
      </StyledListContainer>
    );
  }
}

function applicationGetKey<T extends IApplication>(application: T): string {
  return application.absolutepath;
}

interface IApplicationRowProps<T extends IApplication> {
  application: T;
  onSelect?: (application: T) => void;
  onRemove?: (application: T) => void;
}

function ApplicationRow<T extends IApplication>(props: IApplicationRowProps<T>) {
  const onSelect = useCallback(() => {
    props.onSelect?.(props.application);
  }, [props.onSelect, props.application]);

  const onRemove = useCallback(() => {
    props.onRemove?.(props.application);
  }, [props.onRemove, props.application]);

  return (
    <Cell.CellButton>
      {props.application.icon ? (
        <StyledIcon source={props.application.icon} width={35} height={35} />
      ) : (
        <StyledIconPlaceholder />
      )}
      <StyledCellLabel>{props.application.name}</StyledCellLabel>
      {props.onSelect && (
        <StyledActionIcon
          source="icon-add"
          width={18}
          onClick={onSelect}
          tintColor={colors.white40}
          tintHoverColor={colors.white60}
        />
      )}
      {props.onRemove && (
        <StyledActionIcon
          source="icon-remove"
          width={18}
          onClick={onRemove}
          tintColor={colors.white40}
          tintHoverColor={colors.white60}
        />
      )}
    </Cell.CellButton>
  );
}

interface ISearchBarProps {
  searchTerm: string;
  onSearch: (searchTerm: string) => void;
}

function SearchBar(props: ISearchBarProps) {
  const inputRef = useRef() as React.RefObject<HTMLInputElement>;

  const onInput = useCallback(
    (event: React.FormEvent) => {
      const element = event.target as HTMLInputElement;
      props.onSearch(element.value);
    },
    [props.onSearch],
  );

  const onClear = useCallback(() => {
    props.onSearch('');
    inputRef.current?.blur();
  }, [props.onSearch]);

  return (
    <StyledSearchContainer>
      <StyledSearchInput
        ref={inputRef}
        value={props.searchTerm}
        onInput={onInput}
        placeholder={messages.pgettext('split-tunneling-view', 'Filter...')}
      />
      <StyledSearchIcon source="icon-filter" width={24} tintColor={colors.white60} />
      {props.searchTerm.length > 0 && (
        <StyledClearButton onClick={onClear}>
          <StyledClearIcon source="icon-close" width={18} tintColor={colors.white40} />
        </StyledClearButton>
      )}
    </StyledSearchContainer>
  );
}

function includesSearchTerm(application: IApplication, searchTerm: string) {
  return application.name.toLowerCase().includes(searchTerm.toLowerCase());
}
