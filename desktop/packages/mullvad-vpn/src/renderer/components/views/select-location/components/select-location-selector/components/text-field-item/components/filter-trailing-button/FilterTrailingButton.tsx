import { LocationSelector } from '../../../../../../../../../lib/components/location-selector';
import { useFilterButtonLabel, useHandleFilterButtonClick, useIcon } from './hooks';

export function FilterTrailingButton() {
  const filterButtonLabel = useFilterButtonLabel();
  const icon = useIcon();

  const handleFilterButtonClick = useHandleFilterButtonClick();

  return (
    <LocationSelector.Items.TextFieldItem.TrailingButton
      aria-label={filterButtonLabel}
      visible={true}
      onClick={handleFilterButtonClick}>
      <LocationSelector.Items.TextFieldItem.TrailingButton.Icon icon={icon} />
    </LocationSelector.Items.TextFieldItem.TrailingButton>
  );
}
