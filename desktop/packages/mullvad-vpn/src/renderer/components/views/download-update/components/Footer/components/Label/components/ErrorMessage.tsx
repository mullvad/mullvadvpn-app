import { messages } from '../../../../../../../../../shared/gettext';
import { Flex, LabelTiny } from '../../../../../../../../lib/components';

export function ErrorMessage() {
  return (
    <Flex $gap="small">
      {/*
      TODO: Bring back FeatureIndicator component
       <FeatureIndicator variant="error" /> */}
      <LabelTiny>
        {
          // TRANSLATORS: Label displayed above a progress bar when an error occurred
          messages.pgettext('download-update-view', 'An error occurred')
        }
      </LabelTiny>
    </Flex>
  );
}
