import {
  LocationSelector,
  LocationSelectorSelectedItem,
} from '../../../../../../../lib/components/location-selector';
import { FilterTrailingButtonProvider } from './FilterTrailingButtonContext';
import { useAriaLabel, useHandleFilterButtonClick, useIcon } from './hooks';

export function FilterTrailingButtonImpl() {
  const ariaLabel = useAriaLabel();
  const icon = useIcon();
  const handleFilterButtonClick = useHandleFilterButtonClick();

  return (
    <LocationSelector.Items.TrailingButton
      aria-label={ariaLabel}
      visible={true}
      onClick={handleFilterButtonClick}>
      <LocationSelector.Items.TrailingButton.Icon icon={icon} />
    </LocationSelector.Items.TrailingButton>
  );
}

export type FilterTrailingButtonProps = {
  type: LocationSelectorSelectedItem;
};

export function FilterTrailingButton({ type }: FilterTrailingButtonProps) {
  return (
    <FilterTrailingButtonProvider type={type}>
      <FilterTrailingButtonImpl />
    </FilterTrailingButtonProvider>
  );
}
