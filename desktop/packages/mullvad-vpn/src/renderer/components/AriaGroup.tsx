import React, { useContext, useEffect, useId, useMemo, useState } from 'react';

interface IAriaGroupProps {
  describedId?: string;
  children: React.ReactNode;
}

interface IAriaDescriptionContext {
  describedId: string;
  descriptionId?: string;
  setHasDescription: (value: boolean) => void;
}

const AriaDescriptionContext = React.createContext<IAriaDescriptionContext>({
  get describedId(): string {
    throw new Error('Missing AriaDescriptionContext.Provider');
  },
  setHasDescription(_value) {
    throw new Error('Missing AriaDescriptionContext.Provider');
  },
});

export function AriaDescriptionGroup(props: IAriaGroupProps) {
  const id = useId();
  const [hasDescription, setHasDescription] = useState(false);

  const contextValue = useMemo(
    () => ({
      describedId: props.describedId ?? `${id}-described`,
      descriptionId: hasDescription ? `${id}-description` : undefined,
      setHasDescription,
    }),
    [hasDescription, id, props.describedId],
  );

  return (
    <AriaDescriptionContext.Provider value={contextValue}>
      {props.children}
    </AriaDescriptionContext.Provider>
  );
}

interface IAriaInputContext {
  inputId: string;
  labelId?: string;
  setHasLabel: (value: boolean) => void;
}

const missingAriaInputContextError = new Error('Missing AriaInputContext.Provider');
const AriaInputContext = React.createContext<IAriaInputContext>({
  get inputId(): string {
    throw missingAriaInputContextError;
  },
  setHasLabel() {
    throw missingAriaInputContextError;
  },
});

export const useAriaInputContext = () => useContext(AriaInputContext);

export function AriaInputGroup(props: IAriaGroupProps) {
  const id = useId();

  const [hasLabel, setHasLabel] = useState(false);

  const contextValue = useMemo(
    () => ({
      inputId: `${id}-input`,
      labelId: hasLabel ? `${id}-label` : undefined,
      setHasLabel,
    }),
    [hasLabel, id],
  );

  return (
    <AriaDescriptionGroup describedId={contextValue.inputId}>
      <AriaInputContext.Provider value={contextValue}>{props.children}</AriaInputContext.Provider>
    </AriaDescriptionGroup>
  );
}

interface IAriaElementProps {
  children: React.ReactElement<React.LabelHTMLAttributes<HTMLLabelElement>>;
}

export function AriaInput(props: IAriaElementProps) {
  const { inputId, labelId } = useContext(AriaInputContext);

  return (
    <AriaDescribed>
      {React.cloneElement(props.children, {
        id: inputId,
        'aria-labelledby': labelId,
      })}
    </AriaDescribed>
  );
}

export function AriaLabel(props: IAriaElementProps) {
  const { inputId, labelId, setHasLabel } = useContext(AriaInputContext);

  useEffect(() => {
    setHasLabel(true);
    return () => setHasLabel(false);
  }, [setHasLabel]);

  return React.cloneElement(props.children, {
    id: labelId,
    htmlFor: inputId,
  });
}

export function AriaDescribed(props: IAriaElementProps) {
  const { describedId, descriptionId } = useContext(AriaDescriptionContext);

  return React.cloneElement(props.children, {
    id: describedId,
    'aria-describedby': descriptionId,
  });
}

export function AriaDescription(props: IAriaElementProps) {
  const { descriptionId, setHasDescription } = useContext(AriaDescriptionContext);

  useEffect(() => {
    setHasDescription(true);
    return () => setHasDescription(false);
  }, [setHasDescription]);

  return React.cloneElement(props.children, {
    id: descriptionId,
  });
}

export function AriaDetails(props: IAriaElementProps) {
  const { describedId } = useContext(AriaDescriptionContext);
  return React.cloneElement(props.children, { 'aria-details': describedId });
}
