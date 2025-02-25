import { useCallback } from 'react';
import styled from 'styled-components';

import { urls } from '../../shared/constants';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { useSelector } from '../redux/store';
import { AppNavigationHeader } from './';
import {
  AriaDescribed,
  AriaDescription,
  AriaDescriptionGroup,
  AriaInput,
  AriaInputGroup,
  AriaLabel,
} from './AriaGroup';
import * as Cell from './cell';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';
import { NavigationContainer } from './NavigationContainer';
import { NavigationScrollbars } from './NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  marginBottom: '2px',
});

export default function Support() {
  const { pop } = useHistory();

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader
              title={
                // TRANSLATORS: Title label in navigation bar
                messages.pgettext('support-view', 'Support')
              }
            />

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>{messages.pgettext('support-view', 'Support')}</HeaderTitle>
              </SettingsHeader>

              <StyledContent>
                <Cell.Group>
                  <ProblemReportButton />
                  <FaqButton />
                </Cell.Group>

                <Cell.Group>
                  <BetaProgramSetting />
                </Cell.Group>
              </StyledContent>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

function ProblemReportButton() {
  const history = useHistory();
  const clickHandler = useCallback(() => history.push(RoutePath.problemReport), [history]);

  // TRANSLATORS: Navigation button to the 'Report a problem' help view
  const label = messages.pgettext('support-view', 'Report a problem');

  return (
    <Cell.CellNavigationButton onClick={clickHandler}>
      <Cell.Label>{label}</Cell.Label>
    </Cell.CellNavigationButton>
  );
}

function FaqButton() {
  const isOffline = useSelector((state) => state.connection.isBlocked);
  const { openUrl } = useAppContext();

  const openFaq = useCallback(() => openUrl(urls.faq), [openUrl]);

  return (
    <AriaDescriptionGroup>
      <AriaDescribed>
        <Cell.CellButton disabled={isOffline} onClick={openFaq}>
          <Cell.Label>
            {
              // TRANSLATORS: Link to the webpage
              messages.pgettext('support-view', 'FAQs & Guides')
            }
          </Cell.Label>
          <AriaDescription>
            <Cell.CellTintedIcon
              icon="external"
              aria-label={messages.pgettext('accessibility', 'Opens externally')}
            />
          </AriaDescription>
        </Cell.CellButton>
      </AriaDescribed>
    </AriaDescriptionGroup>
  );
}

function BetaProgramSetting() {
  const isBeta = useSelector((state) => state.version.isBeta);
  const showBetaReleases = useSelector((state) => state.settings.showBetaReleases);
  const { setShowBetaReleases } = useAppContext();

  return (
    <AriaInputGroup>
      <Cell.Container disabled={isBeta}>
        <AriaLabel>
          <Cell.InputLabel>{messages.pgettext('support-view', 'Beta program')}</Cell.InputLabel>
        </AriaLabel>
        <AriaInput>
          <Cell.Switch isOn={showBetaReleases} onChange={setShowBetaReleases} />
        </AriaInput>
      </Cell.Container>
      <Cell.CellFooter>
        <AriaDescription>
          <Cell.CellFooterText>
            {isBeta
              ? messages.pgettext(
                  'support-view',
                  'This option is unavailable while using a beta version.',
                )
              : messages.pgettext(
                  'support-view',
                  'Enable to get notified when new beta versions of the app are released.',
                )}
          </Cell.CellFooterText>
        </AriaDescription>
      </Cell.CellFooter>
    </AriaInputGroup>
  );
}
