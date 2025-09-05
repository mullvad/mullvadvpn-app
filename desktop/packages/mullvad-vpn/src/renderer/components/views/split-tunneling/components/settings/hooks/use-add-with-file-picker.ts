import { useCallback } from 'react';

import { messages } from '../../../../../../../shared/gettext';
import { useFilePicker } from '../../../hooks';
import { useSplitTunnelingContext } from '../../../SplitTunnelingContext';
import { getFilePickerOptionsForPlatform } from '../utils';
import { useAddBrowsedForApplication } from './use-add-browsed-for-application';
import { useScrollToTop } from './use-scroll-to-top';

export function useAddWithFilePicker() {
  const { setBrowsing } = useSplitTunnelingContext();
  const addBrowsedForApplication = useAddBrowsedForApplication();
  const scrollToTop = useScrollToTop();

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
