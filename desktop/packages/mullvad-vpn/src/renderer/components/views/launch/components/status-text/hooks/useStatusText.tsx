import { messages } from '../../../../../../../shared/gettext';
import { BodySmall, Flex, Spinner } from '../../../../../../lib/components';
import { FlexColumn } from '../../../../../../lib/components/flex-column';
import { useUserInterfaceDaemonStatus } from '../../../../../../redux/hooks';

export const useStatusText = () => {
  const { daemonStatus } = useUserInterfaceDaemonStatus();

  let statusMessage = (
    <BodySmall color="whiteAlpha40" textAlign="center" role="alert">
      {
        // TRANSLATORS: Status text app is trying to connect to the system service.
        messages.pgettext('launch-view', 'Connecting to Mullvad system service...')
      }
    </BodySmall>
  );
  if (window.env.platform === 'win32') {
    if (daemonStatus === 'start-requested') {
      statusMessage = (
        <FlexColumn alignItems="center" gap="big">
          <BodySmall color="whiteAlpha40" textAlign="center" role="alert">
            {
              // TRANSLATORS: Status text shown when app is starting.
              messages.pgettext('launch-view', 'Starting up....')
            }
          </BodySmall>
          <Spinner size="medium" />
        </FlexColumn>
      );
    } else {
      statusMessage = (
        <BodySmall color="whiteAlpha40" textAlign="center" role="alert">
          {
            // TRANSLATORS: Status text shown when app fails to start.
            messages.pgettext(
              'launch-view',
              'Failed to start the app, please try again or click “Details” for more info',
            )
          }
        </BodySmall>
      );
    }
  }

  return <Flex justifyContent="center">{statusMessage}</Flex>;
};
