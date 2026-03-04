import { ScrollPositionContextProvider } from './ScrollPositionContext';
import { SelectLocation } from './SelectLocation';
import { SelectLocationViewProvider } from './SelectLocationViewContext';

export function SelectLocationView() {
  return (
    <SelectLocationViewProvider>
      <ScrollPositionContextProvider>
        <SelectLocation />
      </ScrollPositionContextProvider>
    </SelectLocationViewProvider>
  );
}
