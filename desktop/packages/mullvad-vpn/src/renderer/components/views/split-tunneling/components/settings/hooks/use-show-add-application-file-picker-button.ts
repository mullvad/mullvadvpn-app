import { useCanEditSplitTunneling } from './use-can-edit-split-tunneling';

export function useShowAddApplicationFilePickerButton() {
  const canEditSplitTunneling = useCanEditSplitTunneling();

  const showAddApplicationFilePickerButton = canEditSplitTunneling;

  return showAddApplicationFilePickerButton;
}
