import { messages } from '../../../../../../../../shared/gettext';
import { Flex, Icon, LabelTiny } from '../../../../../../../lib/components';
import { Colors } from '../../../../../../../lib/foundations';

export function InstallerReady() {
  return (
    <Flex $gap="tiny" $alignItems="center">
      <Icon icon="checkmark" color={Colors.green} size="small" />
      <LabelTiny>
        {
          // TRANSLATORS: Label displayed above a progress bar when the update is ready to be installed
          messages.pgettext('app-upgrade-view', 'Verification successful! Ready to install.')
        }
      </LabelTiny>
    </Flex>
  );
}
