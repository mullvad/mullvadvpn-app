import { LocationSelector } from '../../../lib/components/location-selector';

export function TestTrailingButton({
  visible,
  activeSettings,
}: {
  visible: boolean;
  activeSettings: boolean;
}) {
  return (
    <LocationSelector.Items.Item.TrailingButton visible={visible}>
      <LocationSelector.Items.Item.TrailingButton.Icon
        icon={activeSettings ? 'filter-active' : 'filter'}
      />
    </LocationSelector.Items.Item.TrailingButton>
  );
}
