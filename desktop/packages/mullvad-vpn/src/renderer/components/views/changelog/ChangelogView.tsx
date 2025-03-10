import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { BodySmall, Container, Flex, TitleBig, TitleLarge } from '../../../lib/components';
import { Colors } from '../../../lib/foundations';
import { useHistory } from '../../../lib/history';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../../';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';

const StyledList = styled(Flex)({
  listStyleType: 'disc',
  paddingLeft: 0,
  li: {
    marginLeft: '1.5em',
  },
});

export const ChangelogView = () => {
  const { pop } = useHistory();
  const changelog = useSelector((state) => state.userInterface.changelog);
  const version = useSelector((state) => state.version.current);

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader title={messages.pgettext('changelog-view', 'What’s new')} />

            <NavigationScrollbars>
              <Flex $flexDirection="column" $gap="large">
                <Container size="4">
                  <TitleBig as={'h1'}>{messages.pgettext('changelog-view', 'What’s new')}</TitleBig>
                </Container>
                <Flex $flexDirection="column" $gap="small">
                  <Container size="4">
                    <TitleLarge as="h2">{version}</TitleLarge>
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
                        {messages.pgettext(
                          'changelog-view',
                          'No updates or changes were made in this release for this platform.',
                        )}
                      </BodySmall>
                    )}
                  </Container>
                </Flex>
              </Flex>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
};
