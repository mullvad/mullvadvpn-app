import { messages } from '../../../../../../shared/gettext';
import { Wizard } from '../../../../../lib/components/wizard';

export function MultihopEntrySetToAutomaticSlide() {
  return (
    <Wizard.Slides.Slide>
      <Wizard.Slides.Slide.Icon icon="info-circle" />
      <Wizard.Slides.Slide.Title>
        {
          // TRANSLATORS: Title shown in one step of a migration wizard.
          messages.pgettext('migrated-settings-view', 'Multihop entry set to "Automatic"')
        }
      </Wizard.Slides.Slide.Title>
      <Wizard.Slides.Slide.Image
        source="entry-set-to-automatic-illustration"
        aria-label={
          // TRANSLATORS: Accessibility label for illustration of multihop entry set to automatic.
          messages.pgettext('accessibility', 'Illustration showing multihop entry set to automatic')
        }
      />
      <Wizard.Slides.Slide.TextGroup>
        <Wizard.Slides.Slide.Text>
          {
            // TRANSLATORS: Instructions shown in one step of a migration wizard.
            messages.pgettext(
              'migrated-settings-view',
              'The multihop mode "Always" now features an "Automatic" location for the entry server selection.',
            )
          }
        </Wizard.Slides.Slide.Text>
        <Wizard.Slides.Slide.Text>
          {
            // TRANSLATORS: Instructions shown in one step of a migration wizard.
            messages.pgettext(
              'migrated-settings-view',
              'When selected, the app automatically picks a random server, prioritizing those closer to the exit location for better performance.',
            )
          }
        </Wizard.Slides.Slide.Text>
      </Wizard.Slides.Slide.TextGroup>
    </Wizard.Slides.Slide>
  );
}
