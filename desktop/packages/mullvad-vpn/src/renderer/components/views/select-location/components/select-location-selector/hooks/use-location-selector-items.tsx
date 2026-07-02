import { useMultihop } from '../../../../../../features/multihop/hooks';
import { SelectLocationSelectorEntryItem, SelectLocationSelectorExitItem } from '../components';
import { useSelectLocationSelectorContext } from '../SelectLocationSelectorContext';

export function useLocationSelectorItems() {
  const { multihop } = useMultihop();
  const { isolatedItem } = useSelectLocationSelectorContext();

  if (multihop) {
    if (isolatedItem === 'entry') {
      return [<SelectLocationSelectorEntryItem key="entry" type="entry" />];
    } else if (isolatedItem === 'exit') {
      return [<SelectLocationSelectorExitItem key="exit" type="exit" />];
    } else {
      return [
        <SelectLocationSelectorEntryItem key="entry" type="entry" />,
        <SelectLocationSelectorExitItem key="exit" type="exit" />,
      ];
    }
  }
  return [<SelectLocationSelectorExitItem key="exit" type="exit" />];
}
