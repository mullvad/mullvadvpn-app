import { useEffect } from 'react';

import { strings } from '../../../../../../shared/constants';
import { useAppContext } from '../../../../../context';
import { Flex, Spinner } from '../../../../../lib/components';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { useAfterTransition } from '../../../../../lib/transition-hooks';
import { useEffectEvent } from '../../../../../lib/utility-hooks';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from '../../../../SettingsHeader';
import { ApplicationSearchBar } from '../application-search-bar';
import { ApplicationSearchNoResult } from '../application-search-no-result';
import {
  HeaderDescription,
  LaunchErrorDialog,
  LinuxApplicationList,
  OpenFilePickerButton,
  UnsupportedDialog,
} from './components';
import { useShowLinuxApplicationList, useShowNoSearchResult, useShowSpinner } from './hooks';
import { LinuxSettingsContextProvider, useLinuxSettingsContext } from './LinuxSettingsContext';

function LinuxSettingsInner() {
  const { getLinuxSplitTunnelingSupported, getLinuxSplitTunnelingApplications } = useAppContext();
  const {
    splitTunnelingSupported,
    searchTerm,
    setApplications,
    setSearchTerm,
    setSplitTunnelingSupported,
  } = useLinuxSettingsContext();
  const runAfterTransition = useAfterTransition();
  const showLinuxApplicationList = useShowLinuxApplicationList();
  const showNoSearchResult = useShowNoSearchResult();
  const showSpinner = useShowSpinner();

  const onMount = useEffectEvent(() => {
    runAfterTransition(async () => {
      const linuxSplitTunnelingSupported = await getLinuxSplitTunnelingSupported();
      setSplitTunnelingSupported(linuxSplitTunnelingSupported);
    });

    runAfterTransition(async () => {
      const applications = await getLinuxSplitTunnelingApplications();
      setApplications(applications);
    });
  });

  // These lint rules are disabled for now because the react plugin for eslint does
  // not understand that useEffectEvent should not be added to the dependency array.
  // Enable these rules again when eslint can lint useEffectEvent properly.
  // eslint-disable-next-line react-compiler/react-compiler
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => void onMount(), []);

  return (
    <>
      <SettingsHeader>
        <HeaderTitle>{strings.splitTunneling}</HeaderTitle>
        <HeaderSubTitle>
          <HeaderDescription />
        </HeaderSubTitle>
      </SettingsHeader>
      <ApplicationSearchBar
        disabled={!splitTunnelingSupported}
        searchTerm={searchTerm}
        onSearch={setSearchTerm}
      />
      {showNoSearchResult && <ApplicationSearchNoResult searchTerm={searchTerm} />}
      <FlexColumn gap="medium">
        {showLinuxApplicationList && <LinuxApplicationList />}
        {showSpinner && (
          <Flex justifyContent="center" margin={{ top: 'large' }}>
            <Spinner size="big" />
          </Flex>
        )}
        <Flex margin={{ horizontal: 'medium', bottom: 'large' }}>
          <OpenFilePickerButton />
        </Flex>
      </FlexColumn>
      <LaunchErrorDialog />
      <UnsupportedDialog />
    </>
  );
}

export function LinuxSettings() {
  return (
    <LinuxSettingsContextProvider>
      <LinuxSettingsInner />
    </LinuxSettingsContextProvider>
  );
}
