import { CustomListLocationsProvider } from './CustomListLocationsContext';
import { LocationsProvider } from './LocationsContext';
import { ScrollPositionContextProvider } from './ScrollPositionContext';
import { SelectLocation } from './SelectLocation';
import { SelectLocationViewProvider } from './SelectLocationViewContext';

export function SelectLocationView() {
  return (
    <SelectLocationViewProvider>
      <ScrollPositionContextProvider>
        <LocationsProvider>
          <CustomListLocationsProvider>
            <SelectLocation />
          </CustomListLocationsProvider>
        </LocationsProvider>
      </ScrollPositionContextProvider>
    </SelectLocationViewProvider>
  );
}
