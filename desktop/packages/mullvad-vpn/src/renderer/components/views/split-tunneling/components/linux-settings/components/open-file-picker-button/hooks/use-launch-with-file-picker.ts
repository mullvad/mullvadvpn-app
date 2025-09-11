import { messages } from '../../../../../../../../../shared/gettext';
import { useFilePicker } from '../../../../../hooks';
import { useSplitTunnelingContext } from '../../../../../SplitTunnelingContext';
import { useLaunchApplication } from '../../../hooks';

export function useLaunchWithFilePicker() {
  const { setBrowsing } = useSplitTunnelingContext();
  const launchApplication = useLaunchApplication();

  const launchWithFilePicker = useFilePicker(
    messages.pgettext('split-tunneling-view', 'Launch'),
    setBrowsing,
    launchApplication,
  );

  return launchWithFilePicker;
}
