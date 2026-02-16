import { CustomListLocationsProvider } from './CustomListLocationsContext';
import { LocationsProvider } from './LocationsContext';
import { RelayListContextProvider } from './RelayListContext';
import { ScrollPositionContextProvider } from './ScrollPositionContext';
import { SelectLocation } from './SelectLocation';
import { SelectLocationViewProvider } from './SelectLocationViewContext';

export function SelectLocationView() {
  return (
    <SelectLocationViewProvider>
      <ScrollPositionContextProvider>
        <RelayListContextProvider>
          <LocationsProvider>
            <CustomListLocationsProvider>
              <SelectLocation />
            </CustomListLocationsProvider>
          </LocationsProvider>
        </RelayListContextProvider>
      </ScrollPositionContextProvider>
    </SelectLocationViewProvider>
  );
}
