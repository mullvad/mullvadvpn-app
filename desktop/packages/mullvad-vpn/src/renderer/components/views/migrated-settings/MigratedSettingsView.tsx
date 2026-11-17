import React from 'react';
import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { useSettingsMigrations } from '../../../features/migration/hooks';
import { EmptyState } from '../../../lib/components/empty-state';
import { View } from '../../../lib/components/view';
import { Wizard } from '../../../lib/components/wizard';
import { colors, spacings } from '../../../lib/foundations';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../../app-navigation-header';
import { StyledScrollable } from '../../CustomScrollbars';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { useMigrationSlides } from './hooks';

const StyledHeader = styled(AppNavigationHeader)`
  background-color: ${colors.darkerBlue50};
`;

const StyledNavigationScrollbars = styled(NavigationScrollbars)`
  ${StyledScrollable} {
    display: flex;
    flex-direction: column;
  }
`;

const StyledViewContainer = styled(View.Container)`
  flex: 1;
`;

const StyledWizardControls = styled(Wizard.Controls)`
  // Apply view margin to controls instead of using View.Content
  // since this does not work well with the double scroll viewers.
  margin-bottom: ${spacings.large};
`;

export function MigratedSettingsView() {
  const { pop } = useHistory();
  const { latestMigration, completeAndClearLatestMigration } = useSettingsMigrations();
  const slides = useMigrationSlides(latestMigration);

  const handleGoBack = React.useCallback(() => {
    pop();
  }, [pop]);

  const handleGotIt = React.useCallback(async () => {
    pop();
    await completeAndClearLatestMigration();
  }, [completeAndClearLatestMigration, pop]);

  const showIndicators = slides && slides.length > 1;

  return (
    <View backgroundColor="darkerBlue50">
      <BackAction action={handleGoBack}>
        <NavigationContainer>
          <StyledHeader
            title={
              // TRANSLATORS: Title for the migrated settings view
              messages.pgettext('migrated-settings-view', 'Migrated settings')
            }
          />
          <StyledNavigationScrollbars>
            <StyledViewContainer horizontalMargin="medium">
              {slides ? (
                <Wizard>
                  <Wizard.Slides>{slides}</Wizard.Slides>
                  <StyledWizardControls>
                    {showIndicators && <Wizard.Controls.Indicators />}
                    <Wizard.Controls.ButtonGroup>
                      <Wizard.Controls.PrevButton />
                      <Wizard.Controls.NextButton />
                      <Wizard.Controls.ExitButton onClick={handleGotIt} />
                    </Wizard.Controls.ButtonGroup>
                  </StyledWizardControls>
                </Wizard>
              ) : (
                <EmptyState>
                  <EmptyState.StatusIcon />
                  <EmptyState.Title>
                    {
                      // TRANSLATORS: Description shown in migration wizard when there are no migrations available.
                      messages.pgettext(
                        'migrated-settings-view',
                        'No settings migrations are available.',
                      )
                    }
                  </EmptyState.Title>
                  <EmptyState.Button onClick={handleGoBack}>
                    <EmptyState.Button.Text>{messages.gettext('Go back')}</EmptyState.Button.Text>
                  </EmptyState.Button>
                </EmptyState>
              )}
            </StyledViewContainer>
          </StyledNavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}
