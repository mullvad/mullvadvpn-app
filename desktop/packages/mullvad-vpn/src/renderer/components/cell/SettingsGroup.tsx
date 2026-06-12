import React, { useCallback, useContext, useEffect, useId, useMemo, useState } from 'react';
import styled from 'styled-components';

import { LabelTinySemiBold } from '../../lib/components';
import { FlexRow } from '../../lib/components/flex-row';
import { Info } from '../info';
import { SettingsRowErrorMessage } from './SettingsRow';

const StyledContainer = styled.div({
  '& ~ &&': {
    marginTop: '20px',
  },
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

  const unsetErrorImpl = useCallback(() => unsetError?.(key), [key, unsetError]);

  useEffect(() => () => unsetErrorImpl(), [unsetErrorImpl]);

  return { reportError, unsetError: unsetErrorImpl };
}

interface SettingsGroupProps {
  title?: string;
  infoMessage?: string | Array<string>;
}

export function SettingsGroup({
  title,
  infoMessage,
  children,
}: React.PropsWithChildren<SettingsGroupProps>) {
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
        {title !== undefined && (
          <FlexRow gap="small" margin={{ left: 'medium', bottom: 'small' }}>
            <LabelTinySemiBold color="whiteAlpha60">{title}</LabelTinySemiBold>
            {infoMessage !== undefined && (
              <Info>
                <Info.Button size="small" />
                <Info.Dialog>
                  {infoMessage instanceof Array ? (
                    infoMessage.map((message, index) => (
                      <Info.Dialog.Text key={index}>{message}</Info.Dialog.Text>
                    ))
                  ) : (
                    <Info.Dialog.Text>{infoMessage}</Info.Dialog.Text>
                  )}
                </Info.Dialog>
              </Info>
            )}
          </FlexRow>
        )}
        <StyledSettingsGroup>{children}</StyledSettingsGroup>
        {Object.values(errors).map((error) => (
          <SettingsRowErrorMessage key={error}>{error}</SettingsRowErrorMessage>
        ))}
      </StyledContainer>
    </settingsGroupContext.Provider>
  );
}
