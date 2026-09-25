import { LocationType } from '../../../../features/locations/types';
import { EntryAutomaticallySelected, LocationLists, LocationSlide } from '../components';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useLocationSlides() {
  const { locationType } = useSelectLocationViewContext();

  switch (locationType) {
    case LocationType.entryAutomatic:
      return (
        <LocationSlide key={'entry-automatic-location-lists'}>
          <EntryAutomaticallySelected />
        </LocationSlide>
      );
    case LocationType.entry:
      return (
        <LocationSlide key={'entry-location-lists'}>
          <LocationLists type={locationType} />
        </LocationSlide>
      );
    case LocationType.exit:
      return (
        <LocationSlide key={'exit-location-lists'}>
          <LocationLists type={locationType} />
        </LocationSlide>
      );
    default:
      return null;
  }
}
