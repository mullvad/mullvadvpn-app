import { useEffect, useRef } from 'react';

import { messages } from '../../../../shared/gettext';
import { useLocale } from '../../../features/client/hooks';
import { Listbox } from '../../../lib/components/listbox';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { CustomScrollbarsRef } from '../../CustomScrollbars';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { HeaderTitle } from '../../SettingsHeader';

export function SelectLanguageView() {
  const { pop } = useHistory();
  const scrollView = useRef<CustomScrollbarsRef>(null);
  const selectedCellRef = useRef<HTMLButtonElement>(null);
  const { preferredLocale, setPreferredLocale, locales } = useLocale();

  const scrollToSelectedCell = () => {
    const ref = selectedCellRef.current;
    const view = scrollView.current;
    if (view && ref) {
      if (ref instanceof HTMLElement) {
        view.scrollToElement(ref, 'middle');
      }
    }
  };

  useEffect(() => {
    scrollToSelectedCell();
  }, []);

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <AppNavigationHeader
            title={
              // TRANSLATORS: Title label in navigation bar
              messages.pgettext('select-language-nav', 'Select language')
            }
          />

          <NavigationScrollbars ref={scrollView}>
            <View.Content>
              <View.Container horizontalMargin="medium" flexDirection="column" gap="medium">
                <HeaderTitle>
                  {messages.pgettext('select-language-nav', 'Select language')}
                </HeaderTitle>
                <Listbox value={preferredLocale} onValueChange={setPreferredLocale}>
                  <Listbox.Options>
                    {locales.map(({ code, name }, idx) => (
                      <Listbox.Option
                        key={code}
                        level={1}
                        value={code}
                        position={idx === 0 ? 'first' : undefined}>
                        <Listbox.Option.Trigger>
                          <Listbox.Option.Item>
                            <Listbox.Option.Label>{name}</Listbox.Option.Label>
                          </Listbox.Option.Item>
                        </Listbox.Option.Trigger>
                      </Listbox.Option>
                    ))}
                  </Listbox.Options>
                </Listbox>
              </View.Container>
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}
