import { CustomListLocationProvider } from './CustomListLocationContext';
import { RelayListContextProvider } from './RelayListContext';
import { ScrollPositionContextProvider } from './ScrollPositionContext';
import { SelectLocation } from './SelectLocation';
import { SelectLocationViewProvider } from './SelectLocationViewContext';

export function SelectLocationView() {
  return (
    <SelectLocationViewProvider>
      <ScrollPositionContextProvider>
        <RelayListContextProvider>
          <CustomListLocationProvider>
            <SelectLocation />
          </CustomListLocationProvider>
        </RelayListContextProvider>
      </ScrollPositionContextProvider>
    </SelectLocationViewProvider>
  );
}
