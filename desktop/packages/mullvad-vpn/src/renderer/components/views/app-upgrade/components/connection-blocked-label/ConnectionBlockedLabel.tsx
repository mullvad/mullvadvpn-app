import { messages } from '../../../../../../shared/gettext';
import { Flex, LabelTinySemiBold } from '../../../../../lib/components';
import { Dot } from '../../../../../lib/components/dot';

export function ConnectionBlockedLabel() {
  return (
    <Flex gap="small" alignItems="baseline">
      <Dot size="small" variant="error" />
      <LabelTinySemiBold>
        {
          // TRANSLATORS: Label displayed when an error occurred due to the connection being blocked
          messages.pgettext(
            'app-upgrade-view',
            'Connection blocked. Try changing server or other settings.',
          )
        }
      </LabelTinySemiBold>
    </Flex>
  );
}
