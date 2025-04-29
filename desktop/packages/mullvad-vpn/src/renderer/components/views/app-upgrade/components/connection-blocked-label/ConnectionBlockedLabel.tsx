import { messages } from '../../../../../../shared/gettext';
import { Flex, LabelTiny } from '../../../../../lib/components';
import { Dot } from '../../../../../lib/components/dot';

export function ConnectionBlockedLabel() {
  return (
    <Flex $gap="small" $alignItems="baseline">
      <Dot size="small" variant="error" />
      <LabelTiny>
        {
          // TRANSLATORS: Label displayed when an error occurred due to the connection being blocked
          messages.pgettext(
            'app-upgrade-view',
            'Connection blocked. Try changing server or other settings',
          )
        }
      </LabelTiny>
    </Flex>
  );
}
