import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { BodySmall, Container, Flex, TitleBig, TitleLarge } from '../../../lib/components';
import { Colors } from '../../../lib/foundations';
import { useHistory } from '../../../lib/history';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { DownloadUpdateViewFooter } from './components';

const StyledList = styled(Flex)`
  list-style-type: disc;
  padding-left: 0;
  li {
    margin-left: 1.5em;
  }
`;

export const DownloadUpdateView = () => {
  const { pop } = useHistory();
  const changelog = useSelector((state) => state.userInterface.changelog);
  const suggestedUpgrade = useSelector((state) => state.version.suggestedUpgrade);

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader
              title={
                // TRANSLATORS: Title in navigation bar
                messages.pgettext('download-update-view', 'Update available')
              }
            />

            <NavigationScrollbars>
              <Flex $flexDirection="column" $gap="large" $padding={{ bottom: 'medium' }}>
                <Container size="4">
                  <TitleBig as={'h2'}>
                    {
                      // TRANSLATORS: Main title for the update available view
                      messages.pgettext('download-update-view', 'Update available')
                    }
                  </TitleBig>
                </Container>
                <Flex $flexDirection="column" $gap="small">
                  <Container size="4">
                    <TitleLarge as="h2">{suggestedUpgrade}</TitleLarge>
                  </Container>
                  <Container size="3" $flexDirection="column">
                    {changelog.length ? (
                      <StyledList as="ul" $flexDirection="column" $gap="medium">
                        {changelog.map((item, i) => (
                          <BodySmall as="li" key={i} color={Colors.white60}>
                            {item}
                          </BodySmall>
                        ))}
                      </StyledList>
                    ) : (
                      <BodySmall color={Colors.white60}>
                        {
                          // TRANSLATORS: Text displayed when there are no updates for this platform in the next version
                          messages.pgettext(
                            'download-update-view',
                            'No updates or changes were made in this release for this platform.',
                          )
                        }
                      </BodySmall>
                    )}
                  </Container>
                </Flex>
              </Flex>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
        <DownloadUpdateViewFooter />
      </Layout>
    </BackAction>
  );
};
