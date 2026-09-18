import { messages } from '../../../../../../shared/gettext';
import { Wizard } from '../../../../../lib/components/wizard';

export function DirectOnlyRemovedSlide() {
  return (
    <Wizard.Slides.Slide>
      <Wizard.Slides.Slide.Icon icon="info-circle" />
      <Wizard.Slides.Slide.Title>
        {
          // TRANSLATORS: Title shown in one step of a migration wizard.
          messages.pgettext('migrated-settings-view', '"Direct only" removed')
        }
      </Wizard.Slides.Slide.Title>
      <Wizard.Slides.Slide.TextGroup>
        <Wizard.Slides.Slide.Text>
          {
            // TRANSLATORS: Instructions shown in one step of a migration wizard.
            messages.pgettext(
              'migrated-settings-view',
              'The DAITA sub-setting "Direct only" has been removed and simplified to avoid blocking connections. Instead with DAITA enabled, you make this option with the multihop setting.',
            )
          }
        </Wizard.Slides.Slide.Text>
        <Wizard.Slides.Slide.Text>
          {
            // TRANSLATORS: Instructions shown in one step of a migration wizard.
            messages.pgettext(
              'migrated-settings-view',
              'When multihop is set to "When needed" the app might use an additional server to make sure you connect to your selected location using DAITA.',
            )
          }
        </Wizard.Slides.Slide.Text>
        <Wizard.Slides.Slide.Text>
          {
            // TRANSLATORS: Instructions shown in one step of a migration wizard.
            messages.pgettext(
              'migrated-settings-view',
              'When multihop is set to "Always" or "Never" you must manually select a DAITA server.',
            )
          }
        </Wizard.Slides.Slide.Text>
      </Wizard.Slides.Slide.TextGroup>
    </Wizard.Slides.Slide>
  );
}
