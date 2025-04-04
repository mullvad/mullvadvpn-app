import { messages } from '../../../../../../../../shared/gettext';
import { Flex, LabelTiny, Spinner } from '../../../../../../../lib/components';

export function VerifyingInstaller() {
  return (
    <Flex $gap="small">
      <Spinner size="small" />
      <LabelTiny>
        {
          // TRANSLATORS: Label displayed above a progress bar when the update is being verified
          messages.pgettext('app-upgrade-view', 'Verifying installer...')
        }
      </LabelTiny>
    </Flex>
  );
}
