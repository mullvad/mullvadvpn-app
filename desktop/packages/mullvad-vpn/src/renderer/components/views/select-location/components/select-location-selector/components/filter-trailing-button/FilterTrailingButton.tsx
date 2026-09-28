import { LocationSelector } from '../../../../../../../lib/components/location-selector';
import {
  FilterTrailingButtonProvider,
  type FilterTrailingButtonProviderProps,
} from './FilterTrailingButtonContext';
import { useFilterButtonLabel, useHandleFilterButtonClick, useIcon } from './hooks';

export function FilterTrailingButtonImpl() {
  const filterButtonLabel = useFilterButtonLabel();
  const icon = useIcon();

  const handleFilterButtonClick = useHandleFilterButtonClick();

  return (
    <LocationSelector.Items.TrailingButton
      aria-label={filterButtonLabel}
      visible={true}
      onClick={handleFilterButtonClick}>
      <LocationSelector.Items.TrailingButton.Icon icon={icon} />
    </LocationSelector.Items.TrailingButton>
  );
}

export function FilterTrailingButton({ id }: FilterTrailingButtonProviderProps) {
  return (
    <FilterTrailingButtonProvider id={id}>
      <FilterTrailingButtonImpl />
    </FilterTrailingButtonProvider>
  );
}
