import React, { useCallback, useContext, useEffect, useId, useMemo, useState } from 'react';
import styled from 'styled-components';

import { colors } from '../../../config.json';
import { measurements, tinyText } from '../common-styles';
import InfoButton from '../InfoButton';
import { SettingsRowErrorMessage } from './SettingsRow';

const StyledContainer = styled.div({
  '& ~ &&': {
    marginTop: '20px',
  },
});

const StyledTitle = styled.h2(tinyText, {
  display: 'flex',
  alignItems: 'center',
  color: colors.white80,
  margin: `0 ${measurements.viewMargin} 8px`,
  lineHeight: '17px',
});

const StyledInfoButton = styled(InfoButton)({
  marginLeft: '6px',
});

export const StyledSettingsGroup = styled.div({});

interface SettingsGroupContext {
  setError?: (key: string, errorMessage: string) => void;
  unsetError?: (key: string) => void;
}

const settingsGroupContext = React.createContext<SettingsGroupContext>({});

export function useSettingsGroupContext() {
  const { setError, unsetError } = useContext(settingsGroupContext);
  const key = useId();

  const reportError = useCallback(
    (errorMessage: string) => {
      setError?.(key, errorMessage);
    },
    [setError, key],
  );

  const unsetErrorImpl = useCallback(() => unsetError?.(key), [unsetError]);

  useEffect(() => () => unsetErrorImpl(), []);

  return { reportError, unsetError: unsetErrorImpl };
}

interface SettingsGroupProps {
  title?: string;
  infoMessage?: string | Array<string>;
}

export function SettingsGroup(props: React.PropsWithChildren<SettingsGroupProps>) {
  const [errors, setErrors] = useState<Record<string, string>>({});

  const setError = useCallback((key: string, errorMessage: string) => {
    setErrors((prevErrors) => ({ ...prevErrors, [key]: errorMessage }));
  }, []);

  const unsetError = useCallback((key: string) => {
    setErrors((prevErrors) => {
      const { [key]: _, ...errors } = prevErrors;
      return errors;
    });
  }, []);

  const contextValue = useMemo(
    () => ({
      setError,
      unsetError,
    }),
    [setError, unsetError],
  );

  return (
    <settingsGroupContext.Provider value={contextValue}>
      <StyledContainer>
        {props.title !== undefined && (
          <StyledTitle>
            {props.title}
            {props.infoMessage !== undefined && (
              <StyledInfoButton size={12} message={props.infoMessage} />
            )}
          </StyledTitle>
        )}
        <StyledSettingsGroup>{props.children}</StyledSettingsGroup>
        {Object.values(errors).map((error) => (
          <SettingsRowErrorMessage key={error}>{error}</SettingsRowErrorMessage>
        ))}
      </StyledContainer>
    </settingsGroupContext.Provider>
  );
}
