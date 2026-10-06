import { EntryAutomaticallySelected, LocationLists, LocationSlide } from '../components';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useLocationSlides() {
  const { locationType } = useSelectLocationViewContext();

  switch (locationType) {
    case 'automaticEntry':
      return (
        <LocationSlide key={'automatic-entry-location-lists'}>
          <EntryAutomaticallySelected />
        </LocationSlide>
      );
    case 'entry':
      return (
        <LocationSlide key={'entry-location-lists'}>
          <LocationLists type={locationType} />
        </LocationSlide>
      );
    case 'exit':
      return (
        <LocationSlide key={'exit-location-lists'}>
          <LocationLists type={locationType} />
        </LocationSlide>
      );
    default:
      return null;
  }
}
