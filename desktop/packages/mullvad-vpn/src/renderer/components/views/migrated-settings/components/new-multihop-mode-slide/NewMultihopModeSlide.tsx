import { messages } from '../../../../../../shared/gettext';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { Wizard } from '../../../../../lib/components/wizard';
import type { FromMultihopMode, ToMultihopMode } from './types';
import { getMultihopMigrationText } from './utils';
import { getToMultihopModeDescription } from './utils/get-to-multihop-mode-description';

export type NewMultihopModeSlideProps = {
  from: FromMultihopMode;
  to: ToMultihopMode;
};

export function NewMultihopModeSlide({ from, to }: NewMultihopModeSlideProps) {
  const migrationText = getMultihopMigrationText(from, to);
  const { label, lines } = getToMultihopModeDescription(to);
  return (
    <Wizard.Slides.Slide>
      <Wizard.Slides.Slide.Icon icon="info-circle" />
      <Wizard.Slides.Slide.Title>
        {
          // TRANSLATORS: Title shown in one step of a migration wizard.
          messages.pgettext('migrated-settings-view', 'New multihop modes')
        }
      </Wizard.Slides.Slide.Title>
      <Wizard.Slides.Slide.TextGroup>
        <Wizard.Slides.Slide.Text>
          {
            // TRANSLATORS: Instructions shown in one step of a migration wizard.
            messages.pgettext(
              'migrated-settings-view',
              'Multihop is now split into three mode: When needed, Always, and Never. This gives you more flexibility with your connection preferences.',
            )
          }
        </Wizard.Slides.Slide.Text>
        <Wizard.Slides.Slide.Text color="white">{migrationText}</Wizard.Slides.Slide.Text>
        <FlexColumn>
          <Wizard.Slides.Slide.Text variant="bodySmallSemibold" color="white">
            {label}
          </Wizard.Slides.Slide.Text>
          {lines.map((line, index) => (
            <Wizard.Slides.Slide.Text key={index}>{line}</Wizard.Slides.Slide.Text>
          ))}
        </FlexColumn>
      </Wizard.Slides.Slide.TextGroup>
    </Wizard.Slides.Slide>
  );
}
