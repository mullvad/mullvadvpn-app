import React, { useContext, useMemo } from 'react';

let groupCounter = 0;
function getNewInputId() {
  return groupCounter++;
}

interface IAriaInputContext {
  inputId?: string;
  labelId?: string;
  descriptionId?: string;
}

const AriaInputContext = React.createContext<IAriaInputContext>({});

interface IAriaInputGroupProps {
  children: React.ReactElement | React.ReactElement[];
}

export function AriaInputGroup(props: IAriaInputGroupProps) {
  const id = useMemo(getNewInputId, []);

  const hasLabel = childrenContainsComponent(props.children, AriaLabel);
  const hasDescription = childrenContainsComponent(props.children, AriaDescription);

  const contextValue = {
    inputId: `${id}-input`,
    labelId: hasLabel ? `${id}-label` : undefined,
    descriptionId: hasDescription ? `${id}-description` : undefined,
  };

  return (
    <AriaInputContext.Provider value={contextValue}>{props.children}</AriaInputContext.Provider>
  );
}

interface IAriaElementProps {
  children: React.ReactElement;
}

export function AriaInput(props: IAriaElementProps) {
  const { inputId, labelId, descriptionId } = useContext(AriaInputContext);

  return React.cloneElement(props.children, {
    id: inputId,
    'aria-labelledby': labelId,
    'aria-describedby': descriptionId,
  });
}

export function AriaLabel(props: IAriaElementProps) {
  const { inputId, labelId } = useContext(AriaInputContext);

  return React.cloneElement(props.children, {
    id: labelId,
    htmlFor: inputId,
  });
}

export function AriaDescription(props: IAriaElementProps) {
  const { descriptionId } = useContext(AriaInputContext);

  return React.cloneElement(props.children, {
    id: descriptionId,
  });
}

function childrenContainsComponent<P>(
  elements: React.ReactNode | React.ReactNodeArray,
  component: React.JSXElementConstructor<P>,
): boolean {
  return React.Children.toArray(elements).some(
    (element) =>
      React.isValidElement(element) &&
      (element.type === component || childrenContainsComponent(element.props.children, component)),
  );
}
