import React, { useContext, useMemo, useState } from 'react';

import useActions from '../../lib/actionsHook';
import { useNormalRelaySettings } from '../../lib/relay-settings-hooks';
import { useSelector } from '../../redux/store';
import userInterface from '../../redux/userinterface/actions';
import { RelayListContextProvider } from './RelayListContext';
import { ScrollPositionContextProvider } from './ScrollPositionContext';
import { LocationType } from './select-location-types';
import SelectLocation from './SelectLocation';

// Context containing data required by different components in the sub tree
interface SelectLocationContext {
  locationType: LocationType;
  setLocationType: (locationType: LocationType) => void;
  searchTerm: string;
  setSearchTerm: (value: string) => void;
}

const selectLocationContext = React.createContext<SelectLocationContext | undefined>(undefined);

export function useSelectLocationContext() {
  return useContext(selectLocationContext)!;
}

export default function SelectLocationContainer() {
  const locationTypeSelector = useSelector((state) => state.userInterface.selectLocationView);
  const { setSelectLocationView } = useActions(userInterface);
  const [searchTerm, setSearchTerm] = useState('');
  const relaySettings = useNormalRelaySettings();
  const bridgeState = useSelector((state) => state.settings.bridgeState);

  const locationType = useMemo(() => {
    const wireguardEnabled = relaySettings?.tunnelProtocol !== 'openvpn';
    const multihopEnabled = wireguardEnabled && relaySettings?.wireguard.useMultihop;

    const openvpnEnabled = relaySettings?.tunnelProtocol !== 'wireguard';
    const bridgeModeEnabled = openvpnEnabled && bridgeState === 'on';

    const allowEntryLocations = multihopEnabled || bridgeModeEnabled;

    if (allowEntryLocations) {
      return locationTypeSelector;
    }
    return LocationType.exit;
  }, [locationTypeSelector, relaySettings, bridgeState]);

  const value = useMemo(
    () => ({
      locationType,
      setLocationType: setSelectLocationView,
      searchTerm,
      setSearchTerm,
    }),
    [locationType, searchTerm, setSelectLocationView],
  );

  return (
    <selectLocationContext.Provider value={value}>
      <ScrollPositionContextProvider>
        <RelayListContextProvider>
          <SelectLocation />
        </RelayListContextProvider>
      </ScrollPositionContextProvider>
    </selectLocationContext.Provider>
  );
}
