import { messages } from '../../../../../../shared/gettext';
import { Wizard } from '../../../../../lib/components/wizard';

export function SeparateFiltersSlide() {
  return (
    <Wizard.Slides.Slide>
      <Wizard.Slides.Slide.Icon icon="info-circle" />
      <Wizard.Slides.Slide.Title>
        {
          // TRANSLATORS: Title shown in one step of a migration wizard.
          messages.pgettext('migrated-settings-view', 'Separate filters')
        }
      </Wizard.Slides.Slide.Title>
      <Wizard.Slides.Slide.Image
        source="separate-filters-illustration"
        aria-label={
          // TRANSLATORS: Instructions shown in one step of a migration wizard.
          messages.pgettext(
            'accessibility',
            'Illustration showing separate filters for entry and exit locations',
          )
        }
      />
      <Wizard.Slides.Slide.TextGroup>
        <Wizard.Slides.Slide.Text>
          {
            // TRANSLATORS: Instructions shown in one step of a migration wizard.
            messages.pgettext(
              'migrated-settings-view',
              'Filters can now be set separately for entry and exit locations',
            )
          }
        </Wizard.Slides.Slide.Text>
        <Wizard.Slides.Slide.Text color="white">
          {
            // TRANSLATORS: Instructions shown in one step of a migration wizard.
            messages.pgettext(
              'migrated-settings-view',
              'Your current filters were applied to both entry and exit locations.',
            )
          }
        </Wizard.Slides.Slide.Text>
      </Wizard.Slides.Slide.TextGroup>
    </Wizard.Slides.Slide>
  );
}
