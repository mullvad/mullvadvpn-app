import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { sprintf } from 'sprintf-js';

import { colors, strings } from '../../config.json';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { formatHtml } from '../lib/html-formatter';
import { IpAddress } from '../lib/ip';
import { useBoolean, useMounted } from '../lib/utilityHooks';
import { useSelector } from '../redux/store';
import Accordion from './Accordion';
import * as AppButton from './AppButton';
import {
  AriaDescribed,
  AriaDescription,
  AriaDescriptionGroup,
  AriaInput,
  AriaInputGroup,
  AriaLabel,
} from './AriaGroup';
import * as Cell from './cell';
import {
  StyledAddCustomDnsButton,
  StyledAddCustomDnsLabel,
  StyledButton,
  StyledContainer,
  StyledCustomDnsFooter,
  StyledLabel,
  StyledRemoveButton,
  StyledRemoveIcon,
} from './CustomDnsSettingsStyles';
import List, { stringValueAsKey } from './List';
import { ModalAlert, ModalAlertType } from './Modal';

export default function CustomDnsSettings() {
  const { setDnsOptions } = useAppContext();
  const dns = useSelector((state) => state.settings.dns);

  const [inputVisible, showInput, hideInput] = useBoolean(false);
  const [invalid, setInvalid, setValid] = useBoolean(false);
  const [confirmAction, setConfirmAction] = useState<() => Promise<void>>();
  const [savingAdd, setSavingAdd] = useState(false);
  const [savingEdit, setSavingEdit] = useState(false);
  const willShowConfirmationDialog = useRef(false);
  const addingLocalIp = useRef(false);
  const manualLocal = window.env.platform === 'win32' || window.env.platform === 'linux';

  const featureAvailable = useMemo(
    () =>
      dns.state === 'custom' ||
      (!dns.defaultOptions.blockAds &&
        !dns.defaultOptions.blockTrackers &&
        !dns.defaultOptions.blockMalware &&
        !dns.defaultOptions.blockAdultContent &&
        !dns.defaultOptions.blockGambling),
    [dns],
  );

  const switchRef = useRef() as React.RefObject<HTMLDivElement>;
  const addButtonRef = useRef() as React.RefObject<HTMLButtonElement>;
  const inputContainerRef = useRef() as React.RefObject<HTMLDivElement>;

  const confirm = useCallback(() => {
    void confirmAction?.();
    setConfirmAction(undefined);
  }, [confirmAction]);
  const abortConfirmation = useCallback(() => {
    setConfirmAction(undefined);
  }, [confirmAction]);

  const setCustomDnsEnabled = useCallback(
    async (enabled: boolean) => {
      if (dns.customOptions.addresses.length > 0) {
        await setDnsOptions({ ...dns, state: enabled ? 'custom' : 'default' });
      }
      if (enabled && dns.customOptions.addresses.length === 0) {
        showInput();
      }
      if (!enabled) {
        hideInput();
      }
    },
    [dns],
  );

  // The input field should be hidden when it loses focus unless something on the same row or the
  // add-button is the new focused element.
  const onInputBlur = useCallback(
    (event?: React.FocusEvent<HTMLTextAreaElement>) => {
      const relatedTarget = event?.relatedTarget as Node | undefined;
      if (
        relatedTarget &&
        (switchRef.current?.contains(relatedTarget) ||
          addButtonRef.current?.contains(relatedTarget) ||
          inputContainerRef.current?.contains(relatedTarget))
      ) {
        event?.target.focus();
      } else if (!willShowConfirmationDialog.current) {
        hideInput();
      }
    },
    [confirmAction, willShowConfirmationDialog],
  );

  const onAdd = useCallback(
    async (address: string) => {
      if (dns.customOptions.addresses.includes(address)) {
        setInvalid();
      } else {
        const add = async () => {
          await setDnsOptions({
            ...dns,
            state: dns.state === 'custom' || inputVisible ? 'custom' : 'default',
            customOptions: {
              addresses: [...dns.customOptions.addresses, address],
            },
          });

          setSavingAdd(true);
          hideInput();
        };

        try {
          const ipAddress = IpAddress.fromString(address);
          addingLocalIp.current = ipAddress.isLocal();
          if (addingLocalIp.current) {
            if (manualLocal) {
              willShowConfirmationDialog.current = true;
              setConfirmAction(() => async () => {
                willShowConfirmationDialog.current = false;
                await add();
              });
            } else {
              await add();
            }
          } else {
            willShowConfirmationDialog.current = true;
            setConfirmAction(() => async () => {
              willShowConfirmationDialog.current = false;
              await add();
            });
          }
        } catch {
          setInvalid();
        }
      }
    },
    [inputVisible, dns, setDnsOptions],
  );

  const onEdit = useCallback(
    (oldAddress: string, newAddress: string) => {
      if (oldAddress !== newAddress && dns.customOptions.addresses.includes(newAddress)) {
        throw new Error('Duplicate address');
      }

      const edit = async () => {
        setSavingEdit(true);

        const addresses = dns.customOptions.addresses.map((address) =>
          oldAddress === address ? newAddress : address,
        );
        await setDnsOptions({
          ...dns,
          customOptions: {
            addresses,
          },
        });
      };

      const ipAddress = IpAddress.fromString(newAddress);
      return new Promise<void>((resolve) => {
        addingLocalIp.current = ipAddress.isLocal();
        if (addingLocalIp.current) {
          if (manualLocal) {
            willShowConfirmationDialog.current = true;
            setConfirmAction(() => async () => {
              willShowConfirmationDialog.current = false;
              await edit();
              resolve();
            });
          } else {
            void edit().then(resolve);
          }
        } else {
          willShowConfirmationDialog.current = true;
          setConfirmAction(() => async () => {
            willShowConfirmationDialog.current = false;
            await edit();
            resolve();
          });
        }
      });
    },
    [dns, setDnsOptions],
  );

  const onRemove = useCallback(
    (address: string) => {
      const addresses = dns.customOptions.addresses.filter((item) => item !== address);
      void setDnsOptions({
        ...dns,
        state: addresses.length > 0 && dns.state === 'custom' ? 'custom' : 'default',
        customOptions: {
          addresses,
        },
      });
    },
    [dns, setDnsOptions],
  );

  useEffect(() => setSavingEdit(false), [dns.customOptions.addresses]);
  useEffect(() => setSavingAdd(false), [dns.customOptions.addresses]);

  const listExpanded = featureAvailable && (dns.state === 'custom' || inputVisible || savingAdd);

  return (
    <>
      <Cell.Container disabled={!featureAvailable}>
        <AriaInputGroup>
          <AriaLabel>
            <Cell.InputLabel>
              {messages.pgettext('vpn-settings-view', 'Use custom DNS server')}
            </Cell.InputLabel>
          </AriaLabel>
          <AriaInput>
            <Cell.Switch
              innerRef={switchRef}
              isOn={dns.state === 'custom' || inputVisible}
              onChange={setCustomDnsEnabled}
            />
          </AriaInput>
        </AriaInputGroup>
      </Cell.Container>
      <Accordion expanded={listExpanded}>
        <Cell.Section role="listbox">
          <List
            items={dns.customOptions.addresses}
            getKey={stringValueAsKey}
            skipAddTransition={true}
            skipRemoveTransition={savingEdit}>
            {(item) => (
              <CellListItem
                onRemove={onRemove}
                onChange={onEdit}
                willShowConfirmationDialog={willShowConfirmationDialog}>
                {item}
              </CellListItem>
            )}
          </List>
        </Cell.Section>

        {inputVisible && (
          <div ref={inputContainerRef}>
            <Cell.RowInput
              placeholder={messages.pgettext('vpn-settings-view', 'Enter IP')}
              onSubmit={onAdd}
              onChange={setValid}
              invalid={invalid}
              paddingLeft={32}
              onBlur={onInputBlur}
              autofocus
            />
          </div>
        )}

        <StyledAddCustomDnsButton
          ref={addButtonRef}
          onClick={showInput}
          disabled={inputVisible}
          tabIndex={-1}>
          <StyledAddCustomDnsLabel tabIndex={-1}>
            {messages.pgettext('vpn-settings-view', 'Add a server')}
          </StyledAddCustomDnsLabel>
          <Cell.Icon
            source="icon-add"
            width={18}
            height={18}
            tintColor={colors.white40}
            tintHoverColor={colors.white60}
            tabIndex={-1}
          />
        </StyledAddCustomDnsButton>
      </Accordion>

      <StyledCustomDnsFooter>
        <Cell.CellFooterText>
          {featureAvailable
            ? messages.pgettext('vpn-settings-view', 'Enable to add at least one DNS server.')
            : formatHtml(
                // TRANSLATORS: This is displayed when either or both of the block ads/trackers settings are
                // TRANSLATORS: turned on which makes the custom DNS setting disabled.
                // TRANSLATORS: Available placeholders:
                // TRANSLATORS: %(preferencesPageName)s - The page title showed on top in the preferences page.
                messages.pgettext(
                  'vpn-settings-view',
                  'Disable all <b>DNS content blockers</b> above to activate this setting.',
                ),
              )}
        </Cell.CellFooterText>
      </StyledCustomDnsFooter>

      <ConfirmationDialog
        isOpen={confirmAction !== undefined}
        isLocal={addingLocalIp}
        confirm={confirm}
        abort={abortConfirmation}
      />
    </>
  );
}

interface ICellListItemProps {
  willShowConfirmationDialog: React.RefObject<boolean>;
  onRemove: (application: string) => void;
  onChange: (value: string, newValue: string) => Promise<void>;
  children: string;
}

function CellListItem(props: ICellListItemProps) {
  const [editing, startEditing, stopEditing] = useBoolean(false);
  const [invalid, setInvalid, setValid] = useBoolean(false);
  const isMounted = useMounted();

  const inputContainerRef = useRef() as React.RefObject<HTMLDivElement>;

  const onRemove = useCallback(() => props.onRemove(props.children), [
    props.onRemove,
    props.children,
  ]);

  const onSubmit = useCallback(
    async (value: string) => {
      if (value === props.children) {
        stopEditing();
      } else {
        try {
          await props.onChange(props.children, value);
          if (isMounted()) {
            stopEditing();
          }
        } catch {
          setInvalid();
        }
      }
    },
    [props.onChange, props.children, invalid],
  );

  const onBlur = useCallback((event?: React.FocusEvent<HTMLTextAreaElement>) => {
    const relatedTarget = event?.relatedTarget as Node | undefined;
    if (relatedTarget && inputContainerRef.current?.contains(relatedTarget)) {
      event?.target.focus();
    } else if (!props.willShowConfirmationDialog.current) {
      stopEditing();
    }
  }, []);

  return (
    <AriaDescriptionGroup>
      {editing ? (
        <div ref={inputContainerRef}>
          <Cell.RowInput
            initialValue={props.children}
            placeholder={messages.pgettext('vpn-settings-view', 'Enter IP')}
            onSubmit={onSubmit}
            onChange={setValid}
            invalid={invalid}
            paddingLeft={32}
            onBlur={onBlur}
            autofocus
          />
        </div>
      ) : (
        <StyledContainer>
          <StyledButton onClick={startEditing}>
            <AriaDescription>
              <StyledLabel>{props.children}</StyledLabel>
            </AriaDescription>
          </StyledButton>
          <AriaDescribed>
            <StyledRemoveButton
              onClick={onRemove}
              aria-label={messages.pgettext('accessibility', 'Remove item')}>
              <StyledRemoveIcon
                source="icon-close"
                width={18}
                height={18}
                tintColor={editing ? colors.black : colors.white40}
              />
            </StyledRemoveButton>
          </AriaDescribed>
        </StyledContainer>
      )}
    </AriaDescriptionGroup>
  );
}

interface IConfirmationDialogProps {
  isOpen: boolean;
  isLocal: React.RefObject<boolean>;
  confirm: () => void;
  abort: () => void;
}

function ConfirmationDialog(props: IConfirmationDialogProps) {
  let message;
  if (props.isLocal.current) {
    message = messages.pgettext(
      'vpn-settings-view',
      'The DNS server you want to add is a private IP. You must ensure that your network interfaces are configured to use it.',
    );
  } else {
    message = sprintf(
      // TRANSLATORS: Available placeholders:
      // TRANSLATORS: %(tunnelProtocol)s - the name of the tunnel protocol setting
      // TRANSLATORS: %(wireguard)s - will be replaced with "WireGuard"
      messages.pgettext(
        'vpn-settings-view',
        'The DNS server you want to add is public and will only work with %(wireguard)s. To ensure that it always works, set the "%(tunnelProtocol)s" (in Advanced settings) to %(wireguard)s.',
      ),
      {
        wireguard: strings.wireguard,
        tunnelProtocol: messages.pgettext('vpn-settings-view', 'Tunnel protocol'),
      },
    );
  }
  return (
    <ModalAlert
      isOpen={props.isOpen}
      type={ModalAlertType.caution}
      buttons={[
        <AppButton.RedButton key="confirm" onClick={props.confirm}>
          {messages.pgettext('vpn-settings-view', 'Add anyway')}
        </AppButton.RedButton>,
        <AppButton.BlueButton key="back" onClick={props.abort}>
          {messages.gettext('Back')}
        </AppButton.BlueButton>,
      ]}
      close={props.abort}
      message={message}></ModalAlert>
  );
}
