import { useEffect } from 'react';

import { useAppContext } from '../../../../../context';
import { Flex, Spinner } from '../../../../../lib/components';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { useAfterTransition } from '../../../../../lib/transition-hooks';
import { useEffectEvent } from '../../../../../lib/utility-hooks';
import { ApplicationSearchBar } from '../application-search-bar';
import { ApplicationSearchNoResult } from '../application-search-no-result';
import {
  Header,
  LaunchErrorDialog,
  LinuxApplicationList,
  OpenFilePickerButton,
} from './components';
import { useShowLinuxApplicationList, useShowNoSearchResult } from './hooks';
import { LinuxSettingsContextProvider, useLinuxSettingsContext } from './LinuxSettingsContext';

function LinuxSettingsInner() {
  const { getLinuxSplitTunnelingApplications } = useAppContext();
  const { searchTerm, setApplications, setSearchTerm } = useLinuxSettingsContext();
  const runAfterTransition = useAfterTransition();
  const showLinuxApplicationList = useShowLinuxApplicationList();
  const showNoSearchResult = useShowNoSearchResult();

  const updateApplications = useEffectEvent(() => {
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
  useEffect(() => void updateApplications(), []);

  return (
    <>
      <Header />
      <ApplicationSearchBar searchTerm={searchTerm} onSearch={setSearchTerm} />
      {showNoSearchResult && <ApplicationSearchNoResult searchTerm={searchTerm} />}
      <FlexColumn $gap="medium">
        {showLinuxApplicationList ? (
          <LinuxApplicationList />
        ) : (
          <Flex $justifyContent="center" $margin={{ top: 'large' }}>
            <Spinner size="big" />
          </Flex>
        )}
        <Flex $margin={{ horizontal: 'medium', bottom: 'large' }}>
          <OpenFilePickerButton />
        </Flex>
      </FlexColumn>
      <LaunchErrorDialog />
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
