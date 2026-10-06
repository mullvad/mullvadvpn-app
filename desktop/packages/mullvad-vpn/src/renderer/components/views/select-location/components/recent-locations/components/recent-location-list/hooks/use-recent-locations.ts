import { useLocationListsContext } from '../../../../location-lists/LocationListsContext';

export function useRecentLocations() {
  const { type, recentEntryLocations, recentExitLocations } = useLocationListsContext();
  if (recentEntryLocations && type === 'entry') {
    return recentEntryLocations;
  } else if (recentExitLocations && type === 'exit') {
    return recentExitLocations;
  }
  return [];
}
