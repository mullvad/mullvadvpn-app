import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useParams } from 'react-router';
import { sprintf } from 'sprintf-js';

import {
  AccessMethod,
  AccessMethodSetting,
  CustomProxy,
  NewAccessMethodSetting,
  RelayProtocol,
  ShadowsocksAccessMethod,
  Socks5LocalAccessMethod,
  Socks5RemoteAccessMethod,
} from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { useScheduler } from '../../shared/scheduler';
import { useAppContext } from '../context';
import { useApiAccessMethodTest } from '../lib/api-access-methods';
import { useHistory } from '../lib/history';
import { IpAddress } from '../lib/ip';
import { useSelector } from '../redux/store';
import * as Cell from './cell';
import { SettingsForm, useSettingsFormSubmittable } from './cell/SettingsForm';
import { SettingsGroup } from './cell/SettingsGroup';
import { SettingsRadioGroup } from './cell/SettingsRadioGroup';
import { SettingsRow } from './cell/SettingsRow';
import { SettingsSelect, SettingsSelectItem } from './cell/SettingsSelect';
import { SettingsNumberInput, SettingsTextInput } from './cell/SettingsTextInput';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';
import { ModalAlert, ModalAlertType } from './Modal';
import { NavigationBar, NavigationContainer, NavigationItems, TitleBarItem } from './NavigationBar';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from './SettingsHeader';
import { StyledContent, StyledNavigationScrollbars, StyledSettingsContent } from './SettingsStyles';
import { SmallButton, SmallButtonGroup } from './SmallButton';

export function EditApiAccessMethod() {
  return (
    <SettingsForm>
      <AccessMethodForm></AccessMethodForm>
    </SettingsForm>
  );
}

function AccessMethodForm() {
  const history = useHistory();
  const { addApiAccessMethod, updateApiAccessMethod } = useAppContext();
  const methods = useSelector((state) => state.settings.apiAccessMethods);

  const [testing, testResult, testApiAccessMethod, resetTestResult] = useApiAccessMethodTest(
    false,
    500,
  );
  const saveScheduler = useScheduler();

  // Use id in url to figure out which method is to be edited. undefined means this is a new method.
  const { id } = useParams<{ id: string | undefined }>();
  // Ugly way of iterating over all access methods, but it works.
  const method = [methods.direct, methods.mullvad_bridges, ...methods.custom].find(
    (method) => method.id === id,
  );

  const updatedMethod = useRef<NewAccessMethodSetting | undefined>(method);
  const updateMethod = useCallback(
    (method: NewAccessMethodSetting) => (updatedMethod.current = method),
    [],
  );

  // Contains form submittability to know whether or not to enable the Add/Save button.
  const formSubmittable = useSettingsFormSubmittable();

  const save = useCallback(() => {
    if (updatedMethod.current !== undefined) {
      resetTestResult();
      if (id === undefined) {
        void addApiAccessMethod(updatedMethod.current);
      } else {
        void updateApiAccessMethod({ ...updatedMethod.current, id });
      }
      history.pop();
    }
  }, [updatedMethod.current, id]);

  const onSave = useCallback(async () => {
    if (
      updatedMethod.current !== undefined &&
      (await testApiAccessMethod(updatedMethod.current as CustomProxy))
    ) {
      // Hide the save dialog after 1.5 seconds.
      saveScheduler.schedule(save, 1500);
    }
  }, [updatedMethod, save, history.pop]);

  const title = getTitle(id !== undefined);
  const subtitle = getSubtitle(id !== undefined);

  return (
    <BackAction action={history.pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <NavigationBar>
              <NavigationItems>
                <TitleBarItem>{title}</TitleBarItem>
              </NavigationItems>
            </NavigationBar>

            <StyledNavigationScrollbars fillContainer>
              <StyledContent>
                <SettingsHeader>
                  <HeaderTitle>{title}</HeaderTitle>
                  <HeaderSubTitle>{subtitle}</HeaderSubTitle>
                </SettingsHeader>

                <StyledSettingsContent>
                  {id !== undefined && method === undefined ? (
                    <span>Failed to open method</span>
                  ) : (
                    <AccessMethodFormImpl method={method} updateMethod={updateMethod} />
                  )}

                  <SmallButtonGroup>
                    <SmallButton onClick={history.pop}>{messages.gettext('Cancel')}</SmallButton>
                    <SmallButton onClick={onSave} disabled={!formSubmittable}>
                      {id === undefined ? messages.gettext('Add') : messages.gettext('Save')}
                    </SmallButton>
                  </SmallButtonGroup>
                </StyledSettingsContent>

                <TestingDialog
                  name={updatedMethod.current?.name ?? ''}
                  newMethod={id === undefined}
                  testing={testing}
                  testResult={testResult}
                  cancel={resetTestResult}
                  save={save}
                />
              </StyledContent>
            </StyledNavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

function getTitle(isNewMethod: boolean) {
  return isNewMethod
    ? messages.pgettext('api-access-methods-view', 'Add method')
    : messages.pgettext('api-access-methods-view', 'Edit method');
}

function getSubtitle(isNewMethod: boolean) {
  return isNewMethod
    ? messages.pgettext('api-access-methods-view', 'Adding a new API access method also tests it.')
    : messages.pgettext('api-access-methods-view', 'Editing an API access method also tests it.');
}

interface TestingDialogProps {
  name: string;
  newMethod: boolean;
  testing: boolean;
  testResult?: boolean;
  cancel: () => void;
  save: () => void;
}

function TestingDialog(props: TestingDialogProps) {
  const type = props.testing
    ? ModalAlertType.loading
    : props.testResult
    ? ModalAlertType.success
    : ModalAlertType.failure;
  const prevType = useRef<ModalAlertType>(type);

  const isOpen = props.testing || props.testResult !== undefined;
  const typeValue = isOpen ? type : prevType.current;

  useEffect(() => {
    if (isOpen) {
      prevType.current = type;
    }
  }, [type]);

  return (
    <ModalAlert
      isOpen={isOpen}
      type={typeValue}
      gridButtons={getTestingDialogButtons(typeValue, props.save, props.cancel)}
      close={props.cancel}
      title={getTestingDialogTitle(typeValue, props.newMethod)}
      message={getTestingDialogSubTitle(typeValue, props.newMethod, props.name)}
    />
  );
}

function getTestingDialogTitle(type: ModalAlertType, newMethod: boolean) {
  switch (type) {
    case ModalAlertType.success:
      return newMethod
        ? messages.pgettext('api-access-methods-view', 'API reachable, adding method…')
        : messages.pgettext('api-access-methods-view', 'API reachable, saving method…');
    case ModalAlertType.failure:
      return newMethod
        ? messages.pgettext('api-access-methods-view', 'API unreachable, add anyway?')
        : messages.pgettext('api-access-methods-view', 'API unreachable, save anyway?');
    default:
    case ModalAlertType.loading:
      return messages.pgettext('api-access-methods-view', 'Testing method...');
  }
}

function getTestingDialogSubTitle(type: ModalAlertType, newMethod: boolean, name: string) {
  switch (type) {
    case ModalAlertType.failure:
      return newMethod
        ? sprintf(
            messages.pgettext(
              'api-access-methods-view',
              'The API could not be reached using the %(name)s method.',
            ),
            { name },
          )
        : messages.pgettext(
            'api-access-methods-view',
            'Clicking “Save” changes the in use method.',
          );
    default:
      return undefined;
  }
}

function getTestingDialogButtons(type: ModalAlertType, save: () => void, cancel: () => void) {
  const saveButton = (
    <SmallButton key="confirm" onClick={save}>
      {messages.gettext('Save')}
    </SmallButton>
  );
  const cancelButton = (
    <SmallButton key="cancel" onClick={cancel}>
      {messages.gettext('Cancel')}
    </SmallButton>
  );
  const disabledCancelButton = (
    <SmallButton key="cancel" onClick={cancel} disabled>
      {messages.gettext('Cancel')}
    </SmallButton>
  );

  switch (type) {
    case ModalAlertType.success:
      return [disabledCancelButton];
    case ModalAlertType.failure:
      return [cancelButton, saveButton];
    case ModalAlertType.loading:
    default:
      return [cancelButton];
  }
}

interface EditApiAccessMethodImplProps {
  method?: AccessMethodSetting;
  updateMethod: (method: NewAccessMethodSetting) => void;
}

function AccessMethodFormImpl(props: EditApiAccessMethodImplProps) {
  // Available method types.
  const types = useMemo<Array<SettingsSelectItem<AccessMethod['type']>>>(
    () => [
      { value: 'shadowsocks', label: 'Shadowsocks' },
      {
        value: 'socks5-remote',
        label: messages.pgettext('api-access-methods-view', 'SOCKS5 remote'),
      },
      {
        value: 'socks5-local',
        label: messages.pgettext('api-access-methods-view', 'SOCKS5 local'),
      },
    ],
    [],
  );
  const [type, setType] = useState(props.method?.type ?? 'shadowsocks');

  // State for the name input.
  const name = useRef(props.method?.name ?? '');
  const updateName = useCallback((value: string) => (name.current = value), []);

  // When the form makes up a valid method the parent is updated.
  const updateMethod = useCallback((value: AccessMethod) => {
    if (name.current !== '') {
      props.updateMethod({ ...value, name: name.current, enabled: true });
    }
  }, []);

  return (
    <>
      <SettingsRow label={messages.gettext('Name')}>
        <SettingsTextInput
          defaultValue={name.current}
          placeholder={messages.pgettext('api-access-methods-view', 'Enter name')}
          onUpdate={updateName}
        />
      </SettingsRow>

      <SettingsRow label={messages.gettext('Type')}>
        <SettingsSelect defaultValue={type} onUpdate={setType} items={types} />
      </SettingsRow>

      {type === 'shadowsocks' && (
        <EditShadowsocks
          onUpdate={updateMethod}
          method={props.method?.type === 'shadowsocks' ? props.method : undefined}
        />
      )}
      {type === 'socks5-remote' && (
        <EditSocks5Remote
          onUpdate={updateMethod}
          method={props.method?.type === 'socks5-remote' ? props.method : undefined}
        />
      )}
      {type === 'socks5-local' && (
        <EditSocks5Local
          onUpdate={updateMethod}
          method={props.method?.type === 'socks5-local' ? props.method : undefined}
        />
      )}
    </>
  );
}

interface EditMethodProps<T> {
  method?: T;
  onUpdate: (method: AccessMethod) => void;
}

function EditShadowsocks(props: EditMethodProps<ShadowsocksAccessMethod>) {
  const [ip, setIp] = useState(props.method?.ip ?? '');
  const [port, setPort] = useState(props.method?.port);
  const [password, setPassword] = useState(props.method?.password ?? '');
  const [cipher, setCipher] = useState(props.method?.cipher);

  const ciphers = useMemo(
    () =>
      [
        { value: 'aes-128-cfb', label: 'aes-128-cfb' },
        { value: 'aes-128-cfb1', label: 'aes-128-cfb1' },
        { value: 'aes-128-cfb8', label: 'aes-128-cfb8' },
        { value: 'aes-128-cfb128', label: 'aes-128-cfb128' },
        { value: 'aes-256-cfb', label: 'aes-256-cfb' },
        { value: 'aes-256-cfb1', label: 'aes-256-cfb1' },
        { value: 'aes-256-cfb8', label: 'aes-256-cfb8' },
        { value: 'aes-256-cfb128', label: 'aes-256-cfb128' },
        { value: 'rc4', label: 'rc4' },
        { value: 'rc4-md5', label: 'rc4-md5' },
        { value: 'chacha20', label: 'chacha20' },
        { value: 'salsa20', label: 'salsa20' },
        { value: 'chacha20-ietf', label: 'chacha20-ietf' },
        { value: 'aes-128-gcm', label: 'aes-128-gcm' },
        { value: 'aes-256-gcm', label: 'aes-256-gcm' },
        { value: 'chacha20-ietf-poly1305', label: 'chacha20-ietf-poly1305' },
        { value: 'xchacha20-ietf-poly1305', label: 'xchacha20-ietf-poly1305' },
        { value: 'aes-128-pmac-siv', label: 'aes-128-pmac-siv' },
        { value: 'aes-256-pmac-siv', label: 'aes-256-pmac-siv' },
      ].sort((a, b) => a.label.localeCompare(b.label)),
    [],
  );

  // Report back to form component with the method values when all required values are set.
  useEffect(() => {
    if (ip !== '' && port !== undefined && cipher !== undefined) {
      props.onUpdate({
        type: 'shadowsocks',
        ip,
        port,
        password,
        cipher,
      });
    }
  }, [ip, port, password, cipher]);

  return (
    <SettingsGroup title={messages.pgettext('api-access-methods-view', 'Server details')}>
      <SettingsRow
        label={messages.pgettext('api-access-methods-view', 'Server')}
        errorMessage={messages.pgettext(
          'api-access-methods-view',
          'Please enter a valid IPv4 or IPv6 address.',
        )}>
        <SettingsTextInput
          value={ip}
          placeholder={messages.pgettext('api-access-methods-view', 'Enter IP')}
          onUpdate={setIp}
          validate={validateIp}
        />
      </SettingsRow>

      <SettingsRow
        label={messages.gettext('Port')}
        errorMessage={messages.pgettext(
          'api-access-methods-view',
          'Please enter a valid remote server port.',
        )}>
        <SettingsNumberInput
          value={port ?? ''}
          placeholder={messages.pgettext('api-access-methods-view', 'Enter port')}
          onUpdate={setPort}
          validate={validatePort}
        />
      </SettingsRow>

      <SettingsRow label={messages.gettext('Password')}>
        <SettingsTextInput
          value={password}
          placeholder={messages.gettext('Optional')}
          onUpdate={setPassword}
          optionalInForm
        />
      </SettingsRow>

      <SettingsRow label={messages.gettext('Cipher')}>
        <SettingsSelect direction="up" defaultValue={cipher} onUpdate={setCipher} items={ciphers} />
      </SettingsRow>
    </SettingsGroup>
  );
}

function EditSocks5Remote(props: EditMethodProps<Socks5RemoteAccessMethod>) {
  const [ip, setIp] = useState(props.method?.ip ?? '');
  const [port, setPort] = useState(props.method?.port);
  const [authentication, setAuthentication] = useState(props.method?.authentication !== undefined);
  const [username, setUsername] = useState(props.method?.authentication?.username ?? '');
  const [password, setPassword] = useState(props.method?.authentication?.password ?? '');

  // Report back to form component with the method values when all required values are set.
  useEffect(() => {
    if (
      ip !== '' &&
      port !== undefined &&
      (!authentication || (username !== '' && password !== ''))
    ) {
      props.onUpdate({
        type: 'socks5-remote',
        ip,
        port,
        authentication: authentication ? { username, password } : undefined,
      });
    }
  }, [ip, port, username, password]);

  return (
    <SettingsGroup title={messages.pgettext('api-access-methods-view', 'Remote Server')}>
      <SettingsRow
        label={messages.pgettext('api-access-methods-view', 'Server')}
        errorMessage={messages.pgettext(
          'api-access-methods-view',
          'Please enter a valid IPv4 or IPv6 address.',
        )}>
        <SettingsTextInput
          value={ip}
          placeholder={messages.pgettext('api-access-methods-view', 'Enter IP')}
          onUpdate={setIp}
          validate={validateIp}
        />
      </SettingsRow>

      <SettingsRow
        label={messages.gettext('Port')}
        errorMessage={messages.pgettext(
          'api-access-methods-view',
          'Please enter a valid remote server port.',
        )}>
        <SettingsNumberInput
          value={port ?? ''}
          placeholder={messages.pgettext('api-access-methods-view', 'Enter port')}
          onUpdate={setPort}
          validate={validatePort}
        />
      </SettingsRow>

      <SettingsRow label={messages.pgettext('api-access-methods-view', 'Authentication')}>
        <Cell.Switch isOn={authentication} onChange={setAuthentication} />
      </SettingsRow>

      {authentication && (
        <>
          <SettingsRow label={messages.gettext('Username')}>
            <SettingsTextInput
              value={username}
              placeholder={messages.gettext('Required')}
              onUpdate={setUsername}
            />
          </SettingsRow>

          <SettingsRow label={messages.gettext('Password')}>
            <SettingsTextInput
              value={password}
              placeholder={messages.gettext('Required')}
              onUpdate={setPassword}
            />
          </SettingsRow>
        </>
      )}
    </SettingsGroup>
  );
}

function EditSocks5Local(props: EditMethodProps<Socks5LocalAccessMethod>) {
  const [remoteIp, setRemoteIp] = useState(props.method?.remoteIp ?? '');
  const [remotePort, setRemotePort] = useState(props.method?.remotePort);
  const [remoteTransportProtocol, setRemoteTransportProtocol] = useState<RelayProtocol>(
    props.method?.remoteTransportProtocol ?? 'tcp',
  );
  const [localPort, setLocalPort] = useState(props.method?.localPort);

  const remoteTransportProtocols = useMemo<Array<SettingsSelectItem<RelayProtocol>>>(
    () => [
      { value: 'tcp', label: 'TCP' },
      { value: 'udp', label: 'UDP' },
    ],
    [],
  );

  useEffect(() => {
    if (remoteIp !== '' && remotePort !== undefined && localPort !== undefined) {
      props.onUpdate({
        type: 'socks5-local',
        remoteIp,
        remotePort,
        remoteTransportProtocol,
        localPort,
      });
    }
  }, [remoteIp, remotePort, localPort, remoteTransportProtocol]);

  return (
    <>
      <SettingsGroup
        title={messages.pgettext('api-access-methods-view', 'Local SOCKS5 server')}
        infoMessage={messages.pgettext(
          'api-access-methods-view',
          'The TCP port where your local SOCKS5 server is listening.',
        )}>
        <SettingsRow
          label={messages.gettext('Port')}
          errorMessage={messages.pgettext(
            'api-access-methods-view',
            'Please enter a valid localhost port.',
          )}>
          <SettingsNumberInput
            value={localPort}
            placeholder={messages.pgettext('api-access-methods-view', 'Enter port')}
            onUpdate={setLocalPort}
            validate={validatePort}
          />
        </SettingsRow>
      </SettingsGroup>

      <SettingsGroup
        title={messages.pgettext('api-access-methods-view', 'Remote Server')}
        infoMessage={[
          messages.pgettext(
            'api-access-methods-view',
            'The app needs the remote server details, where your local SOCKS5 server will forward your traffic.',
          ),
          messages.pgettext(
            'api-access-methods-view',
            'This is needed so our app can allow that traffic in the firewall.',
          ),
        ]}>
        <SettingsRow
          label={messages.pgettext('api-access-methods-view', 'Server')}
          errorMessage={messages.pgettext(
            'api-access-methods-view',
            'Please enter a valid IPv4 or IPv6 address.',
          )}>
          <SettingsTextInput
            value={remoteIp}
            placeholder={messages.pgettext('api-access-methods-view', 'Enter IP')}
            onUpdate={setRemoteIp}
            validate={validateIp}
          />
        </SettingsRow>

        <SettingsRow
          label={messages.gettext('Port')}
          errorMessage={messages.pgettext(
            'api-access-methods-view',
            'Please enter a valid remote server port.',
          )}>
          <SettingsNumberInput
            value={remotePort ?? ''}
            placeholder={messages.pgettext('api-access-methods-view', 'Enter port')}
            onUpdate={setRemotePort}
            validate={validatePort}
          />
        </SettingsRow>

        <SettingsRow label={messages.pgettext('api-access-methods-view', 'Transport protocol')}>
          <SettingsRadioGroup<'tcp' | 'udp'>
            defaultValue={remoteTransportProtocol}
            onUpdate={setRemoteTransportProtocol}
            items={remoteTransportProtocols}
          />
        </SettingsRow>
      </SettingsGroup>
    </>
  );
}

function validateIp(ip: string): boolean {
  try {
    void IpAddress.fromString(ip);
    return true;
  } catch {
    return false;
  }
}

function validatePort(port: number): boolean {
  return port > 0 && port <= 65535;
}
