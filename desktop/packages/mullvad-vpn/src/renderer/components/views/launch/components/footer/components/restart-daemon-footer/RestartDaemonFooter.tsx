import React from 'react';

import { messages } from '../../../../../../../../shared/gettext';
import { useAppContext } from '../../../../../../../context';
import { Button } from '../../../../../../../lib/components';
import { FlexColumn } from '../../../../../../../lib/components/flex-column';
import { useBoolean } from '../../../../../../../lib/utility-hooks';
import { useUserInterfaceDaemonStatus } from '../../../../../../../redux/hooks';
import { TroubleshootingModal } from '../../../troubleshooting-modal';

export function RestartDaemonFooter() {
  const { tryStartDaemon } = useAppContext();
  const { daemonStatus } = useUserInterfaceDaemonStatus();
  const [dialogOpen, showDialog, hideDialog] = useBoolean();

  const handleTryAgain = React.useCallback(() => {
    tryStartDaemon();
  }, [tryStartDaemon]);

  return (
    <>
      <FlexColumn gap="medium">
        <Button onClick={handleTryAgain} disabled={daemonStatus && daemonStatus !== 'stopped'}>
          <Button.Text>
            {
              // TRANSLATORS: Button label for trying to restart the daemon again.
              messages.pgettext('launch-view', 'Try again')
            }
          </Button.Text>
        </Button>
        <Button onClick={showDialog}>
          <Button.Text>
            {
              // TRANSLATORS: Button label for opening dialog with troubleshooting details.
              messages.pgettext('launch-view', 'Details')
            }
          </Button.Text>
        </Button>
      </FlexColumn>
      <TroubleshootingModal isOpen={dialogOpen} onClose={hideDialog} />
    </>
  );
}
