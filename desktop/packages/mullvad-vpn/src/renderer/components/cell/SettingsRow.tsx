import React, { useCallback, useContext, useMemo, useState } from 'react';
import styled from 'styled-components';

import { Icon } from '../../lib/components';
import { Colors } from '../../lib/foundations';
import { AriaInputGroup, AriaLabel } from '../AriaGroup';
import { measurements, smallNormalText, tinyText } from '../common-styles';
import { StyledSettingsGroup, useSettingsGroupContext } from './SettingsGroup';

const StyledSettingsRow = styled.label<{ $invalid: boolean }>((props) => ({
  display: 'flex',
  alignItems: 'center',

  margin: `0 ${measurements.horizontalViewMargin} ${measurements.rowVerticalMargin}`,
  padding: '0 8px',
  minHeight: '36px',
  backgroundColor: Colors.blue60,
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
  borderColor: props.$invalid ? Colors.red : 'transparent',
  outlineColor: props.$invalid ? Colors.red : 'transparent',
  '&&:focus-within': {
    borderColor: props.$invalid ? Colors.red : Colors.white,
    outlineColor: props.$invalid ? Colors.red : Colors.white,
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
  marginLeft: measurements.horizontalViewMargin,
  marginTop: '5px',
  color: Colors.white60,
});

const StyledErrorMessageAlertIcon = styled(Icon)({
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
    [props.errorMessage, reportError, unsetError],
  );

  const contextValue = useMemo(
    () => ({ invalid, setInvalid: setInvalidImpl }),
    [invalid, setInvalidImpl],
  );

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
      <StyledErrorMessageAlertIcon icon="alert-circle" color={Colors.red} size="small" />
      {props.children}
    </StyledSettingsRowErrorMessage>
  );
}
