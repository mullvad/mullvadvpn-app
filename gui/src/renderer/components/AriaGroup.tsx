import React, { useContext, useEffect, useMemo, useState } from 'react';

let groupCounter = 0;
function getNewId() {
  return groupCounter++;
}

interface IAriaControlContext {
  controlledId: string;
}

const AriaControlContext = React.createContext<IAriaControlContext>({
  get controlledId(): string {
    throw new Error('Missing AriaControlContext.Provider');
  },
});

interface IAriaGroupProps {
  describedId?: string;
  children: React.ReactNode;
}

export function AriaControlGroup(props: IAriaGroupProps) {
  const id = useMemo(getNewId, []);
  const contextValue = useMemo(() => ({ controlledId: `${id}-controlled` }), []);

  return (
    <AriaControlContext.Provider value={contextValue}>{props.children}</AriaControlContext.Provider>
  );
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
  const id = useMemo(getNewId, []);
  const [hasDescription, setHasDescription] = useState(false);

  const contextValue = useMemo(
    () => ({
      describedId: props.describedId ?? `${id}-described`,
      descriptionId: hasDescription ? `${id}-description` : undefined,
      setHasDescription,
    }),
    [hasDescription, props.describedId],
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

export function AriaInputGroup(props: IAriaGroupProps) {
  const id = useMemo(getNewId, []);

  const [hasLabel, setHasLabel] = useState(false);

  const contextValue = useMemo(
    () => ({
      inputId: `${id}-input`,
      labelId: hasLabel ? `${id}-label` : undefined,
      setHasLabel,
    }),
    [hasLabel],
  );

  return (
    <AriaDescriptionGroup describedId={contextValue.inputId}>
      <AriaInputContext.Provider value={contextValue}>{props.children}</AriaInputContext.Provider>
    </AriaDescriptionGroup>
  );
}

interface IAriaElementProps {
  children: React.ReactElement;
}

export function AriaControlled(props: IAriaElementProps) {
  const { controlledId } = useContext(AriaControlContext);
  return React.cloneElement(props.children, { id: controlledId });
}

export function AriaControls(props: IAriaElementProps) {
  const { controlledId } = useContext(AriaControlContext);
  return React.cloneElement(props.children, { 'aria-controls': controlledId });
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
  }, []);

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
  }, []);

  return React.cloneElement(props.children, {
    id: descriptionId,
  });
}

export function AriaDetails(props: IAriaElementProps) {
  const { describedId } = useContext(AriaDescriptionContext);
  return React.cloneElement(props.children, { 'aria-details': describedId });
}
