import { useHistory } from '../../../../lib/history';

export function useLocationType() {
  const history = useHistory();
  const filterViewOptions = history.location.state.options?.find(
    (option) => option.type === 'filter-view-location-type',
  );

  const locationType = filterViewOptions?.locationType ?? 'exit';

  return locationType;
}
