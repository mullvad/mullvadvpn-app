import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { Wizard } from '../../../../../lib/components/wizard';
import type { FromMultihopMode, ToMultihopMode } from './types';
import { getFromText } from './utils';
import { getToText } from './utils/get-to-text';

export type NewMultihopModeSlideProps = {
  from: FromMultihopMode;
  to: ToMultihopMode;
};

export function NewMultihopModeSlide({ from, to }: NewMultihopModeSlideProps) {
  const fromText = getFromText(from);
  const { label: toLabel, descriptions } = getToText(to);
  return (
    <Wizard.Slides.Slide>
      <Wizard.Slides.Slide.Icon icon="info-circle" />
      <Wizard.Slides.Slide.Title>
        {
          // TRANSLATORS: Title shown in one step of a migration wizard.
          messages.pgettext('migration-view', 'New multihop modes')
        }
      </Wizard.Slides.Slide.Title>
      <Wizard.Slides.Slide.TextGroup>
        <Wizard.Slides.Slide.Text>
          {
            // TRANSLATORS: Instructions shown in one step of a migration wizard.
            messages.pgettext(
              'migration-view',
              'Multihop is now split into three mode: When needed, Always, and Never. This gives you more flexibility with your connection preferences.',
            )
          }
        </Wizard.Slides.Slide.Text>
        <Wizard.Slides.Slide.Text color="white">
          {sprintf(
            // TRANSLATORS: Instructions shown in one step of a migration wizard.
            // TRANSLATORS: Informs user that their multihop setting has been migrated to a new setting.
            // TRANSLATORS: Available placeholders:
            // TRANSLATORS: %(from)s - The previous multihop setting
            // TRANSLATORS: %(to)s - The new multihop setting
            messages.pgettext(
              'migration-view',
              'Your multihop setting was migrated from "%(from)s" to "%(to)s".',
            ),
            {
              from: fromText,
              to: toLabel,
            },
          )}
        </Wizard.Slides.Slide.Text>
        <FlexColumn>
          <Wizard.Slides.Slide.Text variant="bodySmallSemibold" color="white">
            {toLabel}
          </Wizard.Slides.Slide.Text>
          {descriptions.map((description, index) => (
            <Wizard.Slides.Slide.Text key={index}>{description}</Wizard.Slides.Slide.Text>
          ))}
        </FlexColumn>
      </Wizard.Slides.Slide.TextGroup>
    </Wizard.Slides.Slide>
  );
}
