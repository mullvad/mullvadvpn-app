import { useCallback, useEffect, useMemo, useRef } from 'react';

import { messages } from '../../../../shared/gettext';
import { useAppContext } from '../../../context';
import { Listbox } from '../../../lib/components/listbox';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../..';
import { SelectorItem } from '../../cell/Selector';
import { CustomScrollbarsRef } from '../../CustomScrollbars';
import { BackAction } from '../../KeyboardNavigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';

export function SelectLanguageView() {
  const { pop } = useHistory();
  const { preferredLocale, preferredLocalesList, setPreferredLocale } = usePreferredLocale();
  const scrollView = useRef<CustomScrollbarsRef>(null);
  const selectedCellRef = useRef<HTMLButtonElement>(null);

  const selectLocale = useCallback(
    async (locale: string) => {
      await setPreferredLocale(locale);
      pop();
    },
    [pop, setPreferredLocale],
  );

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
              <SettingsHeader>
                <HeaderTitle>
                  {messages.pgettext('select-language-nav', 'Select language')}
                </HeaderTitle>
              </SettingsHeader>
              <Listbox value={preferredLocale} onValueChange={selectLocale}>
                <Listbox.Options>
                  {preferredLocalesList.map((locale) => (
                    <Listbox.Option key={locale.value} level={1} value={locale.value}>
                      <Listbox.Option.Trigger>
                        <Listbox.Option.Item>
                          <Listbox.Option.Content>
                            <Listbox.Option.Group>
                              <Listbox.Option.Icon icon="checkmark" />
                              <Listbox.Option.Label>{locale.label}</Listbox.Option.Label>
                            </Listbox.Option.Group>
                          </Listbox.Option.Content>
                        </Listbox.Option.Item>
                      </Listbox.Option.Trigger>
                    </Listbox.Option>
                  ))}
                </Listbox.Options>
              </Listbox>
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}

function usePreferredLocale() {
  const preferredLocale = useSelector((state) => state.settings.guiSettings.preferredLocale);

  const { getPreferredLocaleList, setPreferredLocale } = useAppContext();

  const preferredLocalesList: SelectorItem<string>[] = useMemo(() => {
    return [...getPreferredLocaleList().map(({ name, code }) => ({ label: name, value: code }))];
  }, [getPreferredLocaleList]);

  return { preferredLocale, preferredLocalesList, setPreferredLocale };
}
