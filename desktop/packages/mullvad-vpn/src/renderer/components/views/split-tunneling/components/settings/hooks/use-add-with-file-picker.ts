import { useCallback } from 'react';

import { messages } from '../../../../../../../shared/gettext';
import { useFilePicker } from '../../../hooks';
import { useSplitTunnelingContext } from '../../../SplitTunnelingContext';
import { getFilePickerOptionsForPlatform } from '../utils';
import { useAddBrowsedForApplication } from './use-add-browsed-for-application';

export function useAddWithFilePicker() {
  const { scrollToTop, setBrowsing } = useSplitTunnelingContext();
  const addBrowsedForApplication = useAddBrowsedForApplication();

  const filePickerCallback = useFilePicker(
    messages.pgettext('split-tunneling-view', 'Add'),
    setBrowsing,
    addBrowsedForApplication,
    getFilePickerOptionsForPlatform(),
  );

  const addWithFilePicker = useCallback(async () => {
    scrollToTop();
    await filePickerCallback();
  }, [filePickerCallback, scrollToTop]);

  return addWithFilePicker;
}
