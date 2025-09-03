import { useEffect } from 'react';

import { useAppContext } from '../../../../../context';
import { Flex } from '../../../../../lib/components';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { useAfterTransition } from '../../../../../lib/transition-hooks';
import { useEffectEvent } from '../../../../../lib/utility-hooks';
import { NoSearchResult } from '../no-search-result';
import {
  Header,
  LaunchErrorDialog,
  LinuxApplicationList,
  OpenFilePickerButton,
  SearchBar,
} from './components';
import { useShowLinuxApplicationList, useShowNoSearchResult } from './hooks';
import { LinuxSettingsContextProvider, useLinuxSettingsContext } from './LinuxSettingsContext';

function LinuxSettingsInner() {
  const { getLinuxSplitTunnelingApplications } = useAppContext();
  const { searchTerm, setApplications } = useLinuxSettingsContext();
  const runAfterTransition = useAfterTransition();

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

  const showNoSearchResult = useShowNoSearchResult();
  const showLinuxApplicationList = useShowLinuxApplicationList();

  return (
    <>
      <Header />
      <SearchBar />
      {showNoSearchResult && <NoSearchResult searchTerm={searchTerm} />}
      <FlexColumn $gap="medium">
        {showLinuxApplicationList && <LinuxApplicationList />}
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
