import type { LocationSelectorItemType } from '../../../../lib/components/location-selector/components/location-selector-items/types';
import {
  AutomaticEntryItem,
  EntryItem,
  ExitItem,
} from '../components/select-location-selector/components';
import { useShowAutomaticEntryItem } from './use-show-automatic-entry-item';
import { useShowEntryItem } from './use-show-entry-item';
import { useShowExitItem } from './use-show-exit-item';

export function useLocationSelectorItems(
  keyPrefix?: string,
): Partial<Record<LocationSelectorItemType, React.JSX.Element>> {
  const items: Partial<Record<LocationSelectorItemType, React.JSX.Element>> = {};

  const showAutomaticEntryItem = useShowAutomaticEntryItem();
  const showEntryItem = useShowEntryItem();
  const showExitItem = useShowExitItem();

  if (showAutomaticEntryItem) {
    items['entryAutomatic'] = (
      <AutomaticEntryItem key={`${keyPrefix ? `${keyPrefix}-` : ''}entryAutomatic`} />
    );
  }
  if (showEntryItem) {
    items['entry'] = <EntryItem key={`${keyPrefix ? `${keyPrefix}-` : ''}entry`} />;
  }
  if (showExitItem) {
    items['exit'] = <ExitItem key={`${keyPrefix ? `${keyPrefix}-` : ''}exit`} />;
  }

  return items;
}
