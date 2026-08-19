import React from 'react';
import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { useSettingsMigrations } from '../../../features/migration/hooks';
import { EmptyState } from '../../../lib/components/empty-state';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { Wizard } from '../../../lib/components/wizard';
import { colors } from '../../../lib/foundations';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../../app-navigation-header';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { useMigrationSlides } from './hooks';

const StyledHeader = styled(AppNavigationHeader)`
  background-color: ${colors.darkerBlue50};
`;

export function MigratedSettingsView() {
  const { pop } = useHistory();
  const { settingsMigrations, clearSettingsMigrations } = useSettingsMigrations();
  const slides = useMigrationSlides(settingsMigrations);

  const handleGoBack = React.useCallback(() => {
    pop();
  }, [pop]);

  const handleGotIt = React.useCallback(async () => {
    pop();
    await clearSettingsMigrations();
  }, [clearSettingsMigrations, pop]);

  const showIndicators = slides && slides.length > 1;

  return (
    <View backgroundColor="darkerBlue50" aria-label={'migration-view'}>
      <BackAction action={handleGoBack}>
        <NavigationContainer>
          <StyledHeader
            title={
              // TRANSLATORS: Title for the migration view
              messages.pgettext('migrated-settings-view', 'Settings migration')
            }
          />
          <NavigationScrollbars>
            <View.Content aria-label={'content'}>
              <View.Container horizontalMargin="medium" flexGrow={1}>
                {slides ? (
                  <Wizard>
                    <FlexColumn flexGrow={1} gap="medium">
                      <Wizard.Slides>{slides}</Wizard.Slides>
                      <Wizard.Controls>
                        {showIndicators && <Wizard.Controls.Indicators />}
                        <Wizard.Controls.ButtonGroup>
                          <Wizard.Controls.PrevButton />
                          <Wizard.Controls.NextButton />
                          <Wizard.Controls.ExitButton onClick={handleGotIt} />
                        </Wizard.Controls.ButtonGroup>
                      </Wizard.Controls>
                    </FlexColumn>
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
              </View.Container>
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}
