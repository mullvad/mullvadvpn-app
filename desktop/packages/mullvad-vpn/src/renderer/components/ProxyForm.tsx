import React, { useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react';

import {
  CustomProxy,
  NamedCustomProxy,
  RelayProtocol,
  ShadowsocksCustomProxy,
  Socks5LocalCustomProxy,
  Socks5RemoteCustomProxy,
} from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { Button, Flex } from '../lib/components';
import { FlexRow } from '../lib/components/flex-row';
import { Switch } from '../lib/components/switch';
import { IpAddress } from '../lib/ip';
import { useEffectEvent } from '../lib/utility-hooks';
import { SettingsForm, useSettingsFormSubmittable } from './cell/SettingsForm';
import { SettingsGroup } from './cell/SettingsGroup';
import { SettingsRadioGroup } from './cell/SettingsRadioGroup';
import { IndentedRowProps, SettingsRow } from './cell/SettingsRow';
import { SettingsSelect, SettingsSelectItem } from './cell/SettingsSelect';
import {
  SettingsNumberInput,
  SettingsTextInput,
  SettingsTextInputProps,
} from './cell/SettingsTextInput';

interface ProxyFormContext {
  isNew: boolean;
  proxy?: CustomProxy;
  setProxy: (proxy: CustomProxy) => void;
  onSave: () => void;
  onCancel: () => void;
  onDelete?: () => void;
}

const proxyFormContext = React.createContext<ProxyFormContext>({
  get isNew(): boolean {
    throw new Error('Missing ProxyFromContext provider');
  },
  get proxy(): CustomProxy {
    throw new Error('Missing ProxyFromContext provider');
  },
  setProxy(): void {
    throw new Error('Missing ProxyFromContext provider');
  },
  onSave(): void {
    throw new Error('Missing ProxyFromContext provider');
  },
  onCancel(): void {
    throw new Error('Missing ProxyFromContext provider');
  },
  onDelete(): void {
    throw new Error('Missing ProxyFromContext provider');
  },
});

interface ProxyFormContextProviderProps {
  proxy?: CustomProxy;
  onSave: (proxy: CustomProxy) => void;
  onCancel: () => void;
  onDelete?: () => void;
}

function ProxyFormContextProvider(props: React.PropsWithChildren<ProxyFormContextProviderProps>) {
  const { onSave: propsOnSave } = props;

  const [proxy, setProxy] = useState<CustomProxy | undefined>(props.proxy);
  const isNew = props.proxy === undefined;

  const onSave = useCallback(() => {
    if (proxy !== undefined) {
      propsOnSave(proxy);
    }
  }, [proxy, propsOnSave]);

  const value = useMemo(
    () => ({ isNew, proxy, setProxy, onSave, onCancel: props.onCancel, onDelete: props.onDelete }),
    [isNew, proxy, onSave, props.onCancel, props.onDelete],
  );

  return <proxyFormContext.Provider value={value}>{props.children}</proxyFormContext.Provider>;
}

export function ProxyForm(props: ProxyFormContextProviderProps) {
  return (
    <ProxyFormContextProvider {...props}>
      <SettingsForm>
        <ProxyFormInner />
        <ProxyFormButtons />
      </SettingsForm>
    </ProxyFormContextProvider>
  );
}

interface NamedProxyFormContext {
  name?: string;
  setName: (name: string) => void;
}

const namedProxyFormContext = React.createContext<NamedProxyFormContext>({
  get name(): string {
    throw new Error('Missing NamedProxyFromContext provider');
  },
  setName(): void {
    throw new Error('Missing NamedProxyFromContext provider');
  },
});

interface NamedProxyFormContainerProps
  extends Omit<ProxyFormContextProviderProps, 'proxy' | 'onSave'> {
  children?: React.ReactNode;
  proxy?: NamedCustomProxy;
  onSave: (proxy: NamedCustomProxy) => void;
}

export function NamedProxyForm(props: NamedProxyFormContainerProps) {
  const { children, onSave, ...otherProps } = props;

  const [name, setName] = useState<string>(props.proxy?.name ?? '');

  const save = useCallback(
    (proxy: CustomProxy) => {
      if (name !== '') {
        onSave({ ...proxy, name });
      }
    },
    [name, onSave],
  );

  const nameContextValue = useMemo(() => ({ name, setName }), [name]);

  return (
    <namedProxyFormContext.Provider value={nameContextValue}>
      <ProxyFormContextProvider {...otherProps} onSave={save}>
        <SettingsForm>{children}</SettingsForm>
      </ProxyFormContextProvider>
    </namedProxyFormContext.Provider>
  );
}

type ProxyFormNameFieldProps = {
  inputProps?: Partial<SettingsTextInputProps>;
  rowProps?: Partial<IndentedRowProps>;
};

export function ProxyFormNameField(props: ProxyFormNameFieldProps) {
  const { name, setName } = useContext(namedProxyFormContext);

  return (
    <SettingsGroup>
      <SettingsRow label={messages.gettext('Name')} {...props?.rowProps}>
        <SettingsTextInput
          defaultValue={name}
          placeholder={messages.pgettext('api-access-methods-view', 'Enter name')}
          onUpdate={setName}
          {...props?.inputProps}
        />
      </SettingsRow>
    </SettingsGroup>
  );
}

export function ProxyFormButtons() {
  const { isNew, onSave, onCancel, onDelete } = useContext(proxyFormContext);

  // Contains form submittability to know whether or not to enable the Add/Save button.
  const formSubmittable = useSettingsFormSubmittable();
  return (
    <Flex $margin={{ horizontal: 'medium', vertical: 'large' }} $justifyContent="space-between">
      {onDelete !== undefined ? (
        <Button width="fit" variant="destructive" onClick={onDelete}>
          <Button.Text>{messages.gettext('Delete')}</Button.Text>
        </Button>
      ) : (
        <div />
      )}
      <FlexRow $gap="small">
        <Button width="fit" onClick={onCancel}>
          <Button.Text>{messages.gettext('Cancel')}</Button.Text>
        </Button>
        <Button onClick={onSave} disabled={!formSubmittable}>
          <Button.Text>{isNew ? messages.gettext('Add') : messages.gettext('Save')}</Button.Text>
        </Button>
      </FlexRow>
    </Flex>
  );
}

export function ProxyFormInner() {
  const { proxy, setProxy } = useContext(proxyFormContext);

  // Available custom proxies
  const types = useMemo<Array<SettingsSelectItem<CustomProxy['type']>>>(
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
  const [type, setType] = useState(proxy?.type ?? 'shadowsocks');
  const proxyRef = useRef<CustomProxy | undefined>(proxy);

  const updateProxy = useCallback(
    (value: CustomProxy) => {
      proxyRef.current = value;

      // When the form makes up a valid proxy the parent is updated.
      if (proxyRef.current !== undefined) {
        setProxy(proxyRef.current);
      }
    },
    [setProxy],
  );

  return (
    <>
      <SettingsGroup>
        <SettingsRow label={messages.gettext('Type')}>
          <SettingsSelect defaultValue={type} onUpdate={setType} items={types} />
        </SettingsRow>
      </SettingsGroup>

      {type === 'shadowsocks' && (
        <EditShadowsocks
          onUpdate={updateProxy}
          proxy={proxy?.type === 'shadowsocks' ? proxy : undefined}
        />
      )}
      {type === 'socks5-remote' && (
        <EditSocks5Remote
          onUpdate={updateProxy}
          proxy={proxy?.type === 'socks5-remote' ? proxy : undefined}
        />
      )}
      {type === 'socks5-local' && (
        <EditSocks5Local
          onUpdate={updateProxy}
          proxy={proxy?.type === 'socks5-local' ? proxy : undefined}
        />
      )}
    </>
  );
}

interface EditProxyProps<T> {
  proxy?: T;
  onUpdate: (proxy: CustomProxy) => void;
}

function EditShadowsocks(props: EditProxyProps<ShadowsocksCustomProxy>) {
  const [ip, setIp] = useState(props.proxy?.ip ?? '');
  const [port, setPort] = useState(props.proxy?.port);
  const [password, setPassword] = useState(props.proxy?.password ?? '');
  const [cipher, setCipher] = useState(props.proxy?.cipher);

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

  const onUpdate = useEffectEvent(
    (ip: string, port: number | undefined, password: string, cipher: string | undefined) => {
      if (ip !== '' && port !== undefined && cipher !== undefined) {
        props.onUpdate({
          type: 'shadowsocks',
          ip,
          port,
          password,
          cipher,
        });
      }
    },
  );

  // Report back to form component with the proxy values when all required values are set.
  // These lint rules are disabled for now because the react plugin for eslint does
  // not understand that useEffectEvent should not be added to the dependency array.
  // Enable these rules again when eslint can lint useEffectEvent properly.
  // eslint-disable-next-line react-compiler/react-compiler
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => onUpdate(ip, port, password, cipher), [ip, port, password, cipher]);

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
        <SettingsSelect
          data-testid="ciphers"
          direction="up"
          defaultValue={cipher}
          onUpdate={setCipher}
          items={ciphers}
        />
      </SettingsRow>
    </SettingsGroup>
  );
}

function EditSocks5Remote(props: EditProxyProps<Socks5RemoteCustomProxy>) {
  const [ip, setIp] = useState(props.proxy?.ip ?? '');
  const [port, setPort] = useState(props.proxy?.port);
  const [authentication, setAuthentication] = useState(props.proxy?.authentication !== undefined);
  const [username, setUsername] = useState(props.proxy?.authentication?.username ?? '');
  const [password, setPassword] = useState(props.proxy?.authentication?.password ?? '');

  const onUpdate = useEffectEvent(
    (ip: string, port: number | undefined, username: string, password: string) => {
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
    },
  );

  // Report back to form component with the proxy values when all required values are set.
  // These lint rules are disabled for now because the react plugin for eslint does
  // not understand that useEffectEvent should not be added to the dependency array.
  // Enable these rules again when eslint can lint useEffectEvent properly.
  // eslint-disable-next-line react-compiler/react-compiler
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => onUpdate(ip, port, username, password), [ip, port, username, password]);

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
        <Switch checked={authentication} onCheckedChange={setAuthentication}>
          <Switch.Trigger>
            <Switch.Thumb />
          </Switch.Trigger>
        </Switch>
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

function EditSocks5Local(props: EditProxyProps<Socks5LocalCustomProxy>) {
  const [remoteIp, setRemoteIp] = useState(props.proxy?.remoteIp ?? '');
  const [remotePort, setRemotePort] = useState(props.proxy?.remotePort);
  const [remoteTransportProtocol, setRemoteTransportProtocol] = useState<RelayProtocol>(
    props.proxy?.remoteTransportProtocol ?? 'tcp',
  );
  const [localPort, setLocalPort] = useState(props.proxy?.localPort);

  const remoteTransportProtocols = useMemo<Array<SettingsSelectItem<RelayProtocol>>>(
    () => [
      { value: 'tcp', label: 'TCP' },
      { value: 'udp', label: 'UDP' },
    ],
    [],
  );

  const onUpdate = useEffectEvent(
    (
      remoteIp: string,
      remotePort: number | undefined,
      localPort: number | undefined,
      remoteTransportProtocol: RelayProtocol,
    ) => {
      if (remoteIp !== '' && remotePort !== undefined && localPort !== undefined) {
        props.onUpdate({
          type: 'socks5-local',
          remoteIp,
          remotePort,
          remoteTransportProtocol,
          localPort,
        });
      }
    },
  );

  useEffect(
    () => onUpdate(remoteIp, remotePort, localPort, remoteTransportProtocol),
    // These lint rules are disabled for now because the react plugin for eslint does
    // not understand that useEffectEvent should not be added to the dependency array.
    // Enable these rules again when eslint can lint useEffectEvent properly.
    // eslint-disable-next-line react-compiler/react-compiler
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [remoteIp, remotePort, localPort, remoteTransportProtocol],
  );

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
