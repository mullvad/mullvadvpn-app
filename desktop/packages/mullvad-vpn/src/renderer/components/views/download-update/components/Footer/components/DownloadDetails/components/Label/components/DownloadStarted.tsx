import { messages } from '../../../../../../../../../../../shared/gettext';
import { Flex, LabelTiny } from '../../../../../../../../../../lib/components';

export function DownloadStarted() {
  return (
    <Flex $gap="small">
      <LabelTiny>
        {
          // TRANSLATORS: Label displayed above a progress bar when a download is in progress
          messages.pgettext('download-update-view', 'Downloading...')
        }
      </LabelTiny>
    </Flex>
  );
}
