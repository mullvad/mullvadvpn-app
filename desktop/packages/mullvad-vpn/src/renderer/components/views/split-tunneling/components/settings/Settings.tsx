import { useEffect } from 'react';

import { useAppContext } from '../../../../../context';
import { Flex, Spinner } from '../../../../../lib/components';
import { useAfterTransition } from '../../../../../lib/transition-hooks';
import { useEffectEvent } from '../../../../../lib/utility-hooks';
import { ApplicationSearchBar } from '../application-search-bar';
import { ApplicationSearchNoResult } from '../application-search-no-result';
import { AddApplicationFilePickerButton, ApplicationLists, Header } from './components';
import {
  useShowAddApplicationFilePickerButton,
  useShowNoSearchResult,
  useShowSearchBar,
  useShowSpinner,
} from './hooks';
import { useFetchNeedFullDiskPermissions, useShowApplicationLists } from './hooks';
import { SettingsContextProvider, useSettingsContext } from './SettingsContext';

function SettingsInner() {
  const { getSplitTunnelingApplications } = useAppContext();
  const { searchTerm, setApplications, setSearchTerm } = useSettingsContext();
  const fetchNeedFullDiskPermissions = useFetchNeedFullDiskPermissions();
  const runAfterTransition = useAfterTransition();
  const showAddApplicationFilePickerButton = useShowAddApplicationFilePickerButton();
  const showApplicationLists = useShowApplicationLists();
  const showNoSearchResult = useShowNoSearchResult();
  const showSearchBar = useShowSearchBar();
  const showSpinner = useShowSpinner();

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

  // These lint rules are disabled for now because the react plugin for eslint does
  // not understand that useEffectEvent should not be added to the dependency array.
  // Enable these rules again when eslint can lint useEffectEvent properly.
  // eslint-disable-next-line react-compiler/react-compiler
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => void onMount(), []);

  return (
    <>
      <Header />
      {showSpinner && (
        <Flex $justifyContent="center" $margin={{ top: 'large' }}>
          <Spinner size="big" />
        </Flex>
      )}
      {showSearchBar && <ApplicationSearchBar searchTerm={searchTerm} onSearch={setSearchTerm} />}
      {showNoSearchResult && <ApplicationSearchNoResult searchTerm={searchTerm} />}
      <Flex $flexDirection="column" $gap="medium" $margin={{ bottom: 'large' }}>
        {showApplicationLists && <ApplicationLists />}
        {showAddApplicationFilePickerButton && <AddApplicationFilePickerButton />}
      </Flex>
    </>
  );
}

export function Settings() {
  return (
    <SettingsContextProvider>
      <SettingsInner />
    </SettingsContextProvider>
  );
}
