import { useCallback } from 'react';
import styled from 'styled-components';

import { messages } from '../../shared/gettext';
import useActions from '../lib/actionsHook';
import { IconButton } from '../lib/components';
import { Colors } from '../lib/foundations';
import { useHistory } from '../lib/history';
import { useCombinedRefs, useRefCallback, useStyledRef } from '../lib/utility-hooks';
import settingsImportActions from '../redux/settings-import/actions';
import { useSelector } from '../redux/store';
import { AppNavigationHeader } from './';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';

const StyledTextArea = styled.textarea({
  width: '100%',
  flex: 1,
  padding: '13px',
  color: Colors.blue,
});

export default function SettingsTextImport() {
  const { pop } = useHistory();

  const { saveSettingsImportForm } = useActions(settingsImportActions);
  // The textarea value is saved in redux to make it persistent when leaving the view.
  const initialValue = useSelector((state) => state.settingsImport.value);

  const textareaRef = useStyledRef<HTMLTextAreaElement>();
  const onTextareaLoad = useRefCallback((element?: HTMLTextAreaElement) => {
    if (element) {
      element.value = initialValue;
    }
  });

  const combinedTextAreaRef = useCombinedRefs(textareaRef, onTextareaLoad);

  const save = useCallback(() => {
    if (textareaRef.current?.value) {
      saveSettingsImportForm(textareaRef.current.value, true);
    }
    pop();
  }, [pop, saveSettingsImportForm, textareaRef]);

  const back = useCallback(() => {
    if (textareaRef.current) {
      saveSettingsImportForm(textareaRef.current.value, false);
    }
    pop();
  }, [pop, saveSettingsImportForm, textareaRef]);

  return (
    <BackAction action={back}>
      <Layout>
        <SettingsContainer>
          <AppNavigationHeader
            title={
              // TRANSLATORS: Title label in navigation bar
              messages.pgettext('settings-import', 'Import via text')
            }
            titleVisible>
            <AppNavigationHeader.IconButton onClick={save} aria-label={messages.gettext('Save')}>
              <IconButton.Icon icon="checkmark" />
            </AppNavigationHeader.IconButton>
          </AppNavigationHeader>

          <StyledTextArea ref={combinedTextAreaRef} />
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}
