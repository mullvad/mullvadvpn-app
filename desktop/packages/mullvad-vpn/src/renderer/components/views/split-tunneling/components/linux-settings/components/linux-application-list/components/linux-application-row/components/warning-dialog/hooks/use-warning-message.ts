import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../../../../../../shared/gettext';
import { useApplication, useDisabled } from '../../../hooks';

export function useWarningMessage() {
  const application = useApplication();
  const disabled = useDisabled();

  const applicationName = application.name;

  if (disabled) {
    return sprintf(
      messages.pgettext(
        'split-tunneling-view',
        '%(applicationName)s is problematic and can’t be excluded from the VPN tunnel.',
      ),
      {
        applicationName,
      },
    );
  }

  return sprintf(
    messages.pgettext(
      'split-tunneling-view',
      'If it’s already running, close %(applicationName)s before launching it from here. Otherwise it might not be excluded from the VPN tunnel.',
    ),
    {
      applicationName,
    },
  );
}
