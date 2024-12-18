import React, { useCallback, useEffect, useMemo, useState } from 'react';

import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { formatHtml } from '../lib/html-formatter';
import { useBoolean, useMounted, useStyledRef } from '../lib/utility-hooks';
import { useSelector } from '../redux/store';
import Accordion from './Accordion';
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

export default function CustomDnsSettings() {
  const { setDnsOptions } = useAppContext();
  const dns = useSelector((state) => state.settings.dns);

  const [inputVisible, showInput, hideInput] = useBoolean(false);
  const [invalid, setInvalid, setValid] = useBoolean(false);
  const [savingAdd, setSavingAdd] = useState(false);
  const [savingEdit, setSavingEdit] = useState(false);

  const featureAvailable = useMemo(
    () =>
      dns.state === 'custom' ||
      (!dns.defaultOptions.blockAds &&
        !dns.defaultOptions.blockTrackers &&
        !dns.defaultOptions.blockMalware &&
        !dns.defaultOptions.blockAdultContent &&
        !dns.defaultOptions.blockGambling &&
        !dns.defaultOptions.blockSocialMedia),
    [dns],
  );

  const switchRef = useStyledRef<HTMLDivElement>();
  const addButtonRef = useStyledRef<HTMLButtonElement>();
  const inputContainerRef = useStyledRef<HTMLDivElement>();

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
    [dns, hideInput, setDnsOptions, showInput],
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
      } else {
        hideInput();
      }
    },
    [addButtonRef, hideInput, inputContainerRef, switchRef],
  );

  const onAdd = useCallback(
    async (address: string) => {
      if (dns.customOptions.addresses.includes(address)) {
        setInvalid();
      } else {
        try {
          await setDnsOptions({
            ...dns,
            state: dns.state === 'custom' || inputVisible ? 'custom' : 'default',
            customOptions: {
              addresses: [...dns.customOptions.addresses, address],
            },
          });

          setSavingAdd(true);
          hideInput();
        } catch {
          setInvalid();
        }
      }
    },
    [dns, setInvalid, setDnsOptions, inputVisible, hideInput],
  );

  const onEdit = useCallback(
    async (oldAddress: string, newAddress: string) => {
      if (oldAddress !== newAddress && dns.customOptions.addresses.includes(newAddress)) {
        throw new Error('Duplicate address');
      }

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
              <CellListItem onRemove={onRemove} onChange={onEdit}>
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
    </>
  );
}

interface CellListItemPropos {
  onRemove: (application: string) => void;
  onChange: (value: string, newValue: string) => Promise<void>;
  children: string;
}

function CellListItem(props: CellListItemPropos) {
  const { onRemove: propsOnRemove, onChange } = props;

  const [editing, startEditing, stopEditing] = useBoolean(false);
  const [invalid, setInvalid, setValid] = useBoolean(false);
  const isMounted = useMounted();

  const inputContainerRef = useStyledRef<HTMLDivElement>();

  const onRemove = useCallback(
    () => propsOnRemove(props.children),
    [propsOnRemove, props.children],
  );

  const onSubmit = useCallback(
    async (value: string) => {
      if (value === props.children) {
        stopEditing();
      } else {
        try {
          await onChange(props.children, value);
          if (isMounted()) {
            stopEditing();
          }
        } catch {
          setInvalid();
        }
      }
    },
    [props.children, stopEditing, onChange, isMounted, setInvalid],
  );

  const onBlur = useCallback(
    (event?: React.FocusEvent<HTMLTextAreaElement>) => {
      const relatedTarget = event?.relatedTarget as Node | undefined;
      if (relatedTarget && inputContainerRef.current?.contains(relatedTarget)) {
        event?.target.focus();
      } else {
        stopEditing();
      }
    },
    [inputContainerRef, stopEditing],
  );

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
