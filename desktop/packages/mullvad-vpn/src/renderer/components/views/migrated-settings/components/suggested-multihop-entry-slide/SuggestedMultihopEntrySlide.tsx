import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { useMultihop } from '../../../../../features/multihop/hooks';
import { Container } from '../../../../../lib/components';
import { ProgressButton } from '../../../../../lib/components/progress-button';
import { Wizard } from '../../../../../lib/components/wizard';

export function SuggestedMultihopEntrySlide() {
  const [settingAutomaticEntry, setSettingAutomaticEntry] = React.useState(false);
  const { setMultihop, entryLocation } = useMultihop();

  const handleChangeMultihopMode = React.useCallback(async () => {
    setSettingAutomaticEntry(true);

    await setMultihop({
      entryLocation: 'any',
    });

    setSettingAutomaticEntry(false);
  }, [setMultihop]);

  const isEntryAutomatic = entryLocation === 'any';
  const progressButtonStatus = React.useMemo(() => {
    if (isEntryAutomatic) {
      return 'success';
    }
    if (settingAutomaticEntry) {
      return 'loading';
    }
    return 'idle';
  }, [settingAutomaticEntry, isEntryAutomatic]);

  return (
    <Wizard.Slides.Slide>
      <Wizard.Slides.Slide.Icon icon="info-circle" />
      <Wizard.Slides.Slide.Title>
        {
          // TRANSLATORS: Title shown in one step of a migration wizard.
          messages.pgettext('migrated-settings-view', 'Suggested multihop entry')
        }
      </Wizard.Slides.Slide.Title>
      <Wizard.Slides.Slide.TextGroup>
        <Wizard.Slides.Slide.Text>
          {
            // TRANSLATORS: Instructions shown in one step of a migration wizard.
            messages.pgettext(
              'migrated-settings-view',
              'To avoid having to change the entry server manually, we recommend you set the multihop entry server to "Automatic".',
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
        <Wizard.Slides.Slide.Text variant="bodySmallSemibold">
          {
            // TRANSLATORS: Instructions shown in one step of a migration wizard.
            messages.pgettext(
              'migrated-settings-view',
              'Attention: With the "Automatic" location, filters are ignored for the entry server.',
            )
          }
        </Wizard.Slides.Slide.Text>
      </Wizard.Slides.Slide.TextGroup>
      <Container horizontalMargin="small">
        <ProgressButton status={progressButtonStatus} onClick={handleChangeMultihopMode}>
          <ProgressButton.StatusIndicator />
          <ProgressButton.Text>
            {
              // TRANSLATORS: Label for button that changes the multihop entry server to automatic in migration wizard.
              messages.pgettext('migrated-settings-view', 'Set entry to "Automatic"')
            }
          </ProgressButton.Text>
        </ProgressButton>
      </Container>
    </Wizard.Slides.Slide>
  );
}
