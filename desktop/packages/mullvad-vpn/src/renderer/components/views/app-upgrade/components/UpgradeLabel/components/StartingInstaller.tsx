import { messages } from '../../../../../../../shared/gettext';
import { Flex, Icon, LabelTiny } from '../../../../../../lib/components';
import { Colors } from '../../../../../../lib/foundations';

export function StartingInstaller() {
  return (
    <Flex $gap="small">
      <Icon icon="checkmark" color={Colors.green} size="small" />
      <LabelTiny>
        {
          // TRANSLATORS: Label displayed above a progress bar when the update is ready to be installed
          messages.pgettext('app-upgrade-view', 'Verification successful! Starting installer...')
        }
      </LabelTiny>
    </Flex>
  );
}
