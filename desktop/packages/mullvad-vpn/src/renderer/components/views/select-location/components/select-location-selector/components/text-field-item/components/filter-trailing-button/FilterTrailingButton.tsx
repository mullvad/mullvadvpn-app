import { LocationSelector } from '../../../../../../../../../lib/components/location-selector';
import { useTextFieldItemContext } from '../../TextFieldItemContext';
import { useFilterButtonLabel, useHandleFilterButtonClick } from './hooks';

export function FilterTrailingButton() {
  const { id } = useTextFieldItemContext();

  const filterButtonLabel = useFilterButtonLabel();
  const disabled = id === 'entryAutomatic';
  const icon = id === 'entryAutomatic' ? 'filter-overridden' : 'filter';

  const handleFilterButtonClick = useHandleFilterButtonClick();

  return (
    <LocationSelector.Items.TextFieldItem.TextField.TrailingButton
      disabled={disabled}
      aria-label={filterButtonLabel}
      visible={true}
      onClick={handleFilterButtonClick}>
      <LocationSelector.Items.TextFieldItem.TextField.TrailingButton.Icon icon={icon} />
    </LocationSelector.Items.TextFieldItem.TextField.TrailingButton>
  );
}
