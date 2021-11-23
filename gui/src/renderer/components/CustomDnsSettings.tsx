import React, { useCallback, useMemo, useRef, useState } from 'react';
import { sprintf } from 'sprintf-js';
import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { IpAddress } from '../lib/ip';
import { useBoolean } from '../lib/utilityHooks';
import { formatMarkdown } from '../markdown-formatter';
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
  StyledCustomDnsSwitchContainer,
  StyledLabel,
  StyledRemoveButton,
  StyledRemoveIcon,
} from './CustomDnsSettingsStyles';
import { ModalAlert, ModalAlertType } from './Modal';

export default function CustomDnsSettings() {
  const { setDnsOptions } = useAppContext();
  const dns = useSelector((state) => state.settings.dns);

  const [inputVisible, showInput, hideInput] = useBoolean(false);
  const [invalid, setInvalid, setValid] = useBoolean(false);
  const [confirmAction, setConfirmAction] = useState<() => Promise<void>>();
  const willShowConfirmationDialog = useRef(false);

  const featureAvailable = useMemo(
    () =>
      dns.state === 'custom' || (!dns.defaultOptions.blockAds && !dns.defaultOptions.blockTrackers),
    [dns],
  );

  const switchRef = useRef() as React.RefObject<HTMLDivElement>;
  const addButtonRef = useRef() as React.RefObject<HTMLButtonElement>;
  const inputContainerRef = useRef() as React.RefObject<HTMLDivElement>;

  const confirm = useCallback(() => {
    confirmAction?.();
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

          hideInput();
        };

        try {
          const ipAddress = IpAddress.fromString(address);
          if (ipAddress.isLocal()) {
            await add();
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
        if (ipAddress.isLocal()) {
          void edit().then(resolve);
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

  return (
    <>
      <StyledCustomDnsSwitchContainer disabled={!featureAvailable}>
        <AriaInputGroup>
          <AriaLabel>
            <Cell.InputLabel>
              {messages.pgettext('advanced-settings-view', 'Use custom DNS server')}
            </Cell.InputLabel>
          </AriaLabel>
          <AriaInput>
            <Cell.Switch
              ref={switchRef}
              isOn={dns.state === 'custom' || inputVisible}
              onChange={setCustomDnsEnabled}
            />
          </AriaInput>
        </AriaInputGroup>
      </StyledCustomDnsSwitchContainer>
      <Accordion expanded={featureAvailable && (dns.state === 'custom' || inputVisible)}>
        <Cell.Section role="listbox">
          {dns.customOptions.addresses.map((item, i) => {
            return (
              <CellListItem
                key={i}
                onRemove={onRemove}
                onChange={onEdit}
                willShowConfirmationDialog={willShowConfirmationDialog}>
                {item}
              </CellListItem>
            );
          })}
        </Cell.Section>

        {inputVisible && (
          <div ref={inputContainerRef}>
            <Cell.RowInput
              placeholder={messages.pgettext('advanced-settings-view', 'Enter IP')}
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
            {messages.pgettext('advanced-settings-view', 'Add a server')}
          </StyledAddCustomDnsLabel>
          <Cell.Icon
            source="icon-add"
            width={22}
            height={22}
            tintColor={colors.white40}
            tintHoverColor={colors.white60}
            tabIndex={-1}
          />
        </StyledAddCustomDnsButton>
      </Accordion>

      <StyledCustomDnsFooter>
        <Cell.FooterText>
          {featureAvailable ? (
            messages.pgettext('advanced-settings-view', 'Enable to add at least one DNS server.')
          ) : (
            <DisabledMessage />
          )}
        </Cell.FooterText>
      </StyledCustomDnsFooter>

      {confirmAction && <ConfirmationDialog confirm={confirm} abort={abortConfirmation} />}
    </>
  );
}

function DisabledMessage() {
  const blockAdsFeatureName = messages.pgettext('preferences-view', 'Block ads');
  const blockTrackersFeatureName = messages.pgettext('preferences-view', 'Block trackers');
  const preferencesPageName = messages.pgettext('preferences-nav', 'Preferences');

  // TRANSLATORS: This is displayed when either or both of the block ads/trackers settings are
  // TRANSLATORS: turned on which makes the custom DNS setting disabled. The text enclosed in "**"
  // TRANSLATORS: will appear bold.
  // TRANSLATORS: Available placeholders:
  // TRANSLATORS: %(blockAdsFeatureName)s - The name displayed next to the "Block ads" toggle.
  // TRANSLATORS: %(blockTrackersFeatureName)s - The name displayed next to the "Block trackers" toggle.
  // TRANSLATORS: %(preferencesPageName)s - The page title showed on top in the preferences page.
  const customDnsDisabledMessage = messages.pgettext(
    'preferences-view',
    'Disable **%(blockAdsFeatureName)s** and **%(blockTrackersFeatureName)s** (under %(preferencesPageName)s) to activate this setting.',
  );

  return formatMarkdown(
    sprintf(customDnsDisabledMessage, {
      blockAdsFeatureName,
      blockTrackersFeatureName,
      preferencesPageName,
    }),
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
          stopEditing();
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
            placeholder={messages.pgettext('advanced-settings-view', 'Enter IP')}
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
                width={22}
                height={22}
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
  confirm: () => void;
  abort: () => void;
}

function ConfirmationDialog(props: IConfirmationDialogProps) {
  return (
    <ModalAlert
      type={ModalAlertType.info}
      buttons={[
        <AppButton.RedButton key="confirm" onClick={props.confirm}>
          {messages.pgettext('advanced-settings-view', 'Add anyway')}
        </AppButton.RedButton>,
        <AppButton.BlueButton key="back" onClick={props.abort}>
          {messages.gettext('Back')}
        </AppButton.BlueButton>,
      ]}
      close={props.abort}
      message={messages.pgettext(
        'advanced-settings-view',
        'The DNS server you want to add is public and will only work with WireGuard. To ensure that it always works, set the "Tunnel protocol" (in Advanced settings) to WireGuard.',
      )}></ModalAlert>
  );
}
