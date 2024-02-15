import { useCallback } from 'react';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import useActions from '../lib/actionsHook';
import { useHistory } from '../lib/history';
import { useCombinedRefs, useStyledRef } from '../lib/utilityHooks';
import settingsImportActions from '../redux/settings-import/actions';
import { useSelector } from '../redux/store';
import ImageView from './ImageView';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';
import { NavigationBar, NavigationBarButton, NavigationItems, TitleBarItem } from './NavigationBar';

const StyledTextArea = styled.textarea({
  width: '100%',
  flex: 1,
  padding: '13px',
  color: colors.blue,
});

export default function SettingsTextImport() {
  const history = useHistory();

  const { saveSettingsImportForm } = useActions(settingsImportActions);
  // The textarea value is saved in redux to make it persistent when leaving the view.
  const initialValue = useSelector((state) => state.settingsImport.value);

  const textareaRef = useStyledRef<HTMLTextAreaElement>();
  const onTextareaLoad = useCallback((element?: HTMLTextAreaElement) => {
    if (element) {
      element.value = initialValue;
    }
  }, []);

  const combinedTextAreaRef = useCombinedRefs(textareaRef, onTextareaLoad);

  const save = useCallback(() => {
    if (textareaRef.current?.value) {
      saveSettingsImportForm(textareaRef.current.value, true);
    }
    history.pop();
  }, [history]);

  const back = useCallback(() => {
    if (textareaRef.current) {
      saveSettingsImportForm(textareaRef.current.value, false);
    }
    history.pop();
  }, [history]);

  return (
    <BackAction action={back}>
      <Layout>
        <SettingsContainer>
          <NavigationBar alwaysDisplayBarTitle>
            <NavigationItems>
              <TitleBarItem>
                {
                  // TRANSLATORS: Title label in navigation bar
                  messages.pgettext('settings-import', 'Import via text')
                }
              </TitleBarItem>
              <NavigationBarButton onClick={save} aria-label={messages.gettext('Save')}>
                <ImageView
                  source="icon-check"
                  tintColor={colors.white40}
                  tintHoverColor={colors.white60}
                  height={24}
                  width={24}
                />
              </NavigationBarButton>
            </NavigationItems>
          </NavigationBar>

          <StyledTextArea ref={combinedTextAreaRef} />
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}
