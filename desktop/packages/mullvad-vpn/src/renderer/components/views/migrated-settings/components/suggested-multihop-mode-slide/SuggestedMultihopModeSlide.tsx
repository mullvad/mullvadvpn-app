import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { useMultihop } from '../../../../../features/multihop/hooks';
import { Container } from '../../../../../lib/components';
import { ProgressButton } from '../../../../../lib/components/progress-button';
import { Wizard } from '../../../../../lib/components/wizard';

export function SuggestedMultihopModeSlide() {
  const { multihop, setMultihop } = useMultihop();
  const isMultihopWhenNeeded = multihop === 'when-needed';
  const [settingMultihop, setSettingMultihop] = React.useState(false);

  const handleChangeMultihopMode = React.useCallback(async () => {
    setSettingMultihop(true);

    await setMultihop({
      multihop: 'when-needed',
    });

    setSettingMultihop(false);
  }, [setMultihop]);

  const progressButtonStatus = React.useMemo(() => {
    if (settingMultihop) {
      return 'loading';
    }
    if (isMultihopWhenNeeded) {
      return 'success';
    }
    return 'idle';
  }, [isMultihopWhenNeeded, settingMultihop]);

  return (
    <Wizard.Slides.Slide>
      <Wizard.Slides.Slide.Icon icon="info-circle" />
      <Wizard.Slides.Slide.Title>
        {
          // TRANSLATORS: Title shown in one step of a migration wizard.
          messages.pgettext('migrated-settings-view', 'Suggested multihop mode')
        }
      </Wizard.Slides.Slide.Title>
      <Wizard.Slides.Slide.TextGroup>
        <Wizard.Slides.Slide.Text>
          {
            // TRANSLATORS: Instructions shown in one step of a migration wizard.
            messages.pgettext(
              'migrated-settings-view',
              'To avoid getting blocked, we recommend that you set your multihop mode to "When needed".',
            )
          }
        </Wizard.Slides.Slide.Text>
        <Wizard.Slides.Slide.Text>
          {
            // TRANSLATORS: Instructions shown in one step of a migration wizard.
            messages.pgettext(
              'migrated-settings-view',
              'This mode allows the app to automatically multihop through an additional server if needed to ensure your current settings work with your selected location.',
            )
          }
        </Wizard.Slides.Slide.Text>
        <Wizard.Slides.Slide.Text variant="bodySmallSemibold">
          {
            // TRANSLATORS: Instructions shown in one step of a migration wizard.
            messages.pgettext(
              'migrated-settings-view',
              'Attention: In this mode, filters are ignored for the additional server.',
            )
          }
        </Wizard.Slides.Slide.Text>
      </Wizard.Slides.Slide.TextGroup>
      <Container horizontalMargin="small">
        <ProgressButton status={progressButtonStatus} onClick={handleChangeMultihopMode}>
          <ProgressButton.StatusIndicator />
          <ProgressButton.Text>
            {
              // TRANSLATORS: Label for button that changes the multihop mode to "when needed" in migration wizard.
              messages.pgettext('migrated-settings-view', 'Change to "When needed"')
            }
          </ProgressButton.Text>
        </ProgressButton>
      </Container>
    </Wizard.Slides.Slide>
  );
}
