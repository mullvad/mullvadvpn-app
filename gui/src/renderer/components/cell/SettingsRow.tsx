import React, { useCallback, useContext, useMemo, useState } from 'react';
import styled from 'styled-components';

import { colors } from '../../../config.json';
import { AriaInputGroup, AriaLabel } from '../AriaGroup';
import { measurements, smallNormalText, tinyText } from '../common-styles';
import ImageView from '../ImageView';
import { StyledSettingsGroup, useSettingsGroupContext } from './SettingsGroup';

const StyledSettingsRow = styled.label<{ $invalid: boolean }>((props) => ({
  display: 'flex',
  alignItems: 'center',

  margin: `0 ${measurements.viewMargin} ${measurements.rowVerticalMargin}`,
  padding: '0 8px',
  minHeight: '36px',
  backgroundColor: colors.blue60,
  borderRadius: '4px',

  [`${StyledSettingsGroup} &&`]: {
    marginBottom: 0,
  },

  [`${StyledSettingsGroup} &&:not(:last-child)`]: {
    marginBottom: '1px',
    borderBottomLeftRadius: 0,
    borderBottomRightRadius: 0,
  },

  [`${StyledSettingsGroup} &&:not(:first-child)`]: {
    borderTopLeftRadius: 0,
    borderTopRightRadius: 0,
  },

  borderWidth: '1px',
  outlineWidth: '1px',
  borderStyle: 'solid',
  outlineStyle: 'solid',
  borderColor: props.$invalid ? colors.red : 'transparent',
  outlineColor: props.$invalid ? colors.red : 'transparent',
  '&&:focus-within': {
    borderColor: props.$invalid ? colors.red : colors.white,
    outlineColor: props.$invalid ? colors.red : colors.white,
  },
}));

const StyledLabel = styled.div(smallNormalText, {
  display: 'flex',
  flex: 1,
  margin: '4px 0',
});

const StyledInputContainer = styled.div({
  display: 'flex',
  flex: 1,
  justifyContent: 'end',
});

const StyledSettingsRowErrorMessage = styled.div(tinyText, {
  display: 'flex',
  alignItems: 'center',
  marginLeft: measurements.viewMargin,
  marginTop: '5px',
  color: colors.white60,
});

const StyledErrorMessageAlertIcon = styled(ImageView)({
  marginRight: '5px',
});

interface SettingsRowContext {
  invalid: boolean;
  setInvalid: (invalid: boolean) => void;
}

// Keeps track of input validity to show red border if an invalid value is provided.
const settingsRowContext = React.createContext<SettingsRowContext>({
  invalid: false,
  setInvalid: (_invalid: boolean) => {
    throw new Error('setInvalid not defined');
  },
});

export function useSettingsRowContext() {
  return useContext(settingsRowContext);
}

interface IndentedRowProps {
  label: string;
  infoMessage?: string | Array<string>;
  errorMessage?: string;
}

export function SettingsRow(props: React.PropsWithChildren<IndentedRowProps>) {
  const { reportError, unsetError } = useSettingsGroupContext();
  const [invalid, setInvalid] = useState(false);

  const setInvalidImpl = useCallback(
    (invalid: boolean) => {
      setInvalid(invalid);
      if (reportError !== undefined && props.errorMessage !== undefined && invalid) {
        reportError(props.errorMessage);
      } else if (unsetError !== undefined && !invalid) {
        unsetError?.();
      }
    },
    [reportError, unsetError],
  );

  const contextValue = useMemo(() => ({ invalid, setInvalid: setInvalidImpl }), [invalid]);

  return (
    <settingsRowContext.Provider value={contextValue}>
      <AriaInputGroup>
        <AriaLabel>
          <StyledSettingsRow $invalid={invalid}>
            <StyledLabel>{props.label}</StyledLabel>
            <StyledInputContainer>{props.children}</StyledInputContainer>
          </StyledSettingsRow>
        </AriaLabel>
        {reportError === undefined && invalid && props.errorMessage && (
          <SettingsRowErrorMessage>{props.errorMessage}</SettingsRowErrorMessage>
        )}
      </AriaInputGroup>
    </settingsRowContext.Provider>
  );
}

export function SettingsRowErrorMessage(props: React.PropsWithChildren) {
  return (
    <StyledSettingsRowErrorMessage>
      <StyledErrorMessageAlertIcon source="icon-alert" tintColor={colors.red} width={12} />
      {props.children}
    </StyledSettingsRowErrorMessage>
  );
}
