import { useCallback, useState } from 'react';
import styled from 'styled-components';

import { ImportConfigError, ImportConfigResponse } from '../../../../shared/daemon-rpc-types';
import { messages } from '../../../../shared/gettext';
import { useAppContext } from '../../../context';
import { Button, Flex, Spinner, Text } from '../../../lib/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { colors } from '../../../lib/foundations';
import { IconBadge } from '../../../lib/icon-badge';
import { useSelector } from '../../../redux/store';
import { AppMainHeader } from '../../app-main-header';

const StyledTextArea = styled.textarea({
  width: '100%',
  minHeight: '120px',
  padding: '10px',
  borderRadius: '4px',
  border: 'none',
  resize: 'none',
  fontFamily: 'monospace',
  fontSize: '12px',
  color: colors.blue,
});

type ImportState =
  | { type: 'idle' }
  | { type: 'importing' }
  | { type: 'imported' }
  | { type: 'failed'; error: ImportConfigError };

export function ImportConfigView() {
  const { importWireguardConfigText, importWireguardConfigFile, showOpenDialog } = useAppContext();

  const isPerformingPostUpgrade = useSelector(
    (state) => state.userInterface.isPerformingPostUpgrade,
  );
  const tunnelState = useSelector((state) => state.connection.status);
  const showBlockMessage =
    tunnelState.state === 'error' ||
    (tunnelState.state === 'disconnected' && tunnelState.lockedDown);

  const [configText, setConfigText] = useState('');
  const [state, setState] = useState<ImportState>({ type: 'idle' });

  const allowInteraction = state.type !== 'importing' && state.type !== 'imported';

  const handleResponse = useCallback((response: ImportConfigResponse) => {
    if (response.type === 'success') {
      // The config contains a private key, don't keep it around any longer than necessary.
      setConfigText('');
      setState({ type: 'imported' });
    } else {
      setState({ type: 'failed', error: response });
    }
  }, []);

  const importText = useCallback(async () => {
    setState({ type: 'importing' });
    handleResponse(await importWireguardConfigText(configText));
  }, [configText, handleResponse, importWireguardConfigText]);

  const importFile = useCallback(async () => {
    const file = await showOpenDialog({
      properties: ['openFile'],
      buttonLabel: messages.gettext('Import'),
      filters: [
        {
          // TRANSLATORS: Name of the file type filter in the file selection dialog when importing a
          // TRANSLATORS: WireGuard configuration file.
          name: messages.pgettext('import-config-view', 'WireGuard config'),
          extensions: ['conf', 'txt'],
        },
        { name: messages.pgettext('import-config-view', 'All files'), extensions: ['*'] },
      ],
    });
    const path = file.filePaths[0];
    if (file.canceled || path === undefined) {
      return;
    }

    setState({ type: 'importing' });
    handleResponse(await importWireguardConfigFile(path));
  }, [handleResponse, importWireguardConfigFile, showOpenDialog]);

  const onTextChange = useCallback((event: React.ChangeEvent<HTMLTextAreaElement>) => {
    setConfigText(event.target.value);
    setState((state) => (state.type === 'failed' ? { type: 'idle' } : state));
  }, []);

  return (
    <View>
      <AppMainHeader>
        <AppMainHeader.SettingsButton disabled={!allowInteraction} />
      </AppMainHeader>
      <View.Content>
        <View.Container flexDirection="column" horizontalMargin="medium" justifyContent="center">
          <FlexColumn gap="medium">
            <Flex justifyContent="center">
              <StatusIcon
                state={state}
                busy={isPerformingPostUpgrade === true}
                showBlockMessage={showBlockMessage}
              />
            </Flex>

            <View.Container
              gap="large"
              horizontalMargin="small"
              justifyContent="center"
              flexDirection="column">
              <FlexColumn gap="small">
                <Text as="h1" variant="titleBig" aria-live="polite">
                  {title(state)}
                </Text>
                <Text
                  variant="labelTiny"
                  color={state.type === 'failed' ? 'red' : 'whiteAlpha60'}
                  aria-live="polite"
                  data-testid="import-config-subtitle">
                  {subtitle(state)}
                </Text>
                {state.type === 'failed' && state.error.message ? (
                  <Text variant="labelTiny" color="whiteAlpha60" data-testid="import-config-detail">
                    {state.error.message}
                  </Text>
                ) : null}
              </FlexColumn>

              <FlexColumn gap="small">
                <StyledTextArea
                  value={configText}
                  onChange={onTextChange}
                  disabled={!allowInteraction}
                  spellCheck={false}
                  autoCapitalize="off"
                  autoCorrect="off"
                  data-testid="import-config-text"
                  aria-label={messages.pgettext(
                    'import-config-view',
                    'Paste the contents of the WireGuard config here',
                  )}
                  placeholder="[Interface]&#10;PrivateKey = ...&#10;Address = 10.x.x.x/32,fc00:...&#10;..."
                />
                <Button
                  color="success"
                  onClick={importText}
                  disabled={!allowInteraction || configText.trim() === ''}
                  data-testid="import-config-text-button">
                  <Button.Text>
                    {
                      // TRANSLATORS: Button label for importing the WireGuard config text that was
                      // TRANSLATORS: pasted into the text field.
                      messages.pgettext('import-config-view', 'Import pasted config')
                    }
                  </Button.Text>
                </Button>
                <Button
                  onClick={importFile}
                  disabled={!allowInteraction}
                  data-testid="import-config-file-button">
                  <Button.Text>
                    {
                      // TRANSLATORS: Button label for selecting a WireGuard config file to import.
                      messages.pgettext('import-config-view', 'Import config file')
                    }
                  </Button.Text>
                </Button>
              </FlexColumn>
            </View.Container>
          </FlexColumn>
        </View.Container>
      </View.Content>
    </View>
  );
}

function StatusIcon(props: { state: ImportState; busy: boolean; showBlockMessage: boolean }) {
  if (props.busy || props.state.type === 'importing') {
    return <Spinner size="big" />;
  }

  switch (props.state.type) {
    case 'failed':
      return <IconBadge state="negative" />;
    case 'imported':
      return <IconBadge state="positive" />;
    default:
      return props.showBlockMessage ? <IconBadge state="negative" /> : null;
  }
}

function title(state: ImportState): string {
  switch (state.type) {
    case 'importing':
      return messages.pgettext('import-config-view', 'Importing...');
    case 'imported':
      return messages.pgettext('import-config-view', 'Config imported');
    case 'failed':
      return messages.pgettext('import-config-view', 'Import failed');
    default:
      return messages.pgettext('import-config-view', 'Import WireGuard config');
  }
}

function subtitle(state: ImportState): string {
  switch (state.type) {
    case 'importing':
      return messages.pgettext('import-config-view', 'Please wait');
    case 'imported':
      return messages.pgettext('import-config-view', 'Ready to connect');
    case 'failed':
      return errorString(state.error);
    default:
      return messages.pgettext(
        'import-config-view',
        'Generate a WireGuard config on mullvad.net and import it here. It is used to connect to every server.',
      );
  }
}

function errorString(error: ImportConfigError): string {
  switch (error.error) {
    case 'invalid-config':
      // TRANSLATORS: Error message shown when the imported text is not a usable WireGuard config.
      return messages.pgettext('import-config-view', 'This is not a valid WireGuard config');
    case 'file-read':
      // TRANSLATORS: Error message shown when the selected config file could not be read.
      return messages.pgettext('import-config-view', 'The file could not be read');
    case 'communication':
      // TRANSLATORS: Error message shown when the app fails to talk to the background service.
      return messages.pgettext('import-config-view', 'Could not reach the Mullvad VPN service');
  }
}
